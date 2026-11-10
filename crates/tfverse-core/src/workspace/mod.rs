//! The project layer: confined file access, config, stack detection, the lockfile and the object cache.

mod config;
mod detect;
mod fs;
mod lock;
mod store;

use std::io::{self, ErrorKind};
use std::path::Path;
use std::sync::Arc;

pub use config::{
	ConfigError, ExternalPatcher, HostAuth, ProjectConfig, RegistryAliases, RegistryConfig, SigningConfig, UserConfig,
};
pub use detect::ProjectModel;
pub(crate) use detect::detect;
pub use fs::{DiskFs, EntryKind, Fs, MemoryFs};
pub(crate) use lock::instance_id;
pub use lock::{FileRecord, Instance, LockError, Lockfile, PackRecord, PatchRecord};
pub(crate) use store::ObjectStore;
pub use store::StoreError;

use crate::brand;
use crate::model::RelPath;

const STATE_GITIGNORE: &str = "# tfverse local state: cache and history. Safe to delete.\n*\n";

/// A project confined to its root: tfverse never goes through symlinks or files in the way.
#[derive(Clone, Debug)]
pub(crate) struct Workspace {
	fs: Arc<dyn Fs>,
}

/// Why a project file operation failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkspaceError {
	/// An I/O error.
	#[error("could not access `{path}`: {source}")]
	Io {
		/// The path involved.
		path: String,
		/// The cause.
		source: io::Error,
	},
	/// tfverse never reads or writes through symlinks.
	#[error("`{0}` is a symlink; tfverse never reads or writes through symlinks")]
	Symlink(RelPath),
	/// A folder where a file is expected.
	#[error("`{0}` is a folder, not a file")]
	NotAFile(RelPath),
	/// A file where a folder is expected.
	#[error("`{0}` is a file, but a folder is needed there")]
	NotADir(RelPath),
}

impl Workspace {
	pub(crate) fn new(fs: Arc<dyn Fs>) -> Self {
		Self { fs }
	}

	pub(crate) fn open(root: &Path) -> Result<Self, WorkspaceError> {
		let disk = DiskFs::open(root).map_err(|source| WorkspaceError::Io {
			path: root.display().to_string(),
			source,
		})?;
		Ok(Self::new(Arc::new(disk)))
	}

	/// A path inside `.tfverse/`.
	pub(crate) fn state_path(rest: &str) -> RelPath {
		RelPath::trusted(format!("{}/{rest}", brand::STATE_DIR))
	}

	/// Creates `.tfverse/` with a `.gitignore` that keeps it out of version control.
	pub(crate) fn init_state(&self) -> Result<(), WorkspaceError> {
		self.create_new(&Self::state_path(".gitignore"), STATE_GITIGNORE.as_bytes())?;
		Ok(())
	}

	/// Reads a file, or `None` when it does not exist.
	pub(crate) fn read(&self, path: &RelPath) -> Result<Option<Vec<u8>>, WorkspaceError> {
		match self.entry(path)? {
			None => Ok(None),
			Some(EntryKind::File) => self.fs.read(path).map(Some).map_err(io_error(path)),
			Some(EntryKind::Dir) => Err(WorkspaceError::NotAFile(path.clone())),
			Some(EntryKind::Symlink) => Err(WorkspaceError::Symlink(path.clone())),
		}
	}

	/// Whether a file exists at `path`.
	pub(crate) fn exists(&self, path: &RelPath) -> Result<bool, WorkspaceError> {
		Ok(self.entry(path)? == Some(EntryKind::File))
	}

	/// Writes a file atomically, creating parent folders; returns the folders it created.
	pub(crate) fn write(&self, path: &RelPath, bytes: &[u8]) -> Result<Vec<RelPath>, WorkspaceError> {
		let created = self.prepare_file(path)?;
		self.fs.write(path, bytes).map_err(io_error(path))?;
		Ok(created)
	}

	/// Creates a file unless something is already there; returns whether it did.
	pub(crate) fn create_new(&self, path: &RelPath, bytes: &[u8]) -> Result<bool, WorkspaceError> {
		self.prepare_file(path)?;
		match self.fs.create_new(path, bytes) {
			Ok(()) => Ok(true),
			Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(false),
			Err(source) => Err(io_error(path)(source)),
		}
	}

	/// Removes a file; returns whether it existed.
	pub(crate) fn remove(&self, path: &RelPath) -> Result<bool, WorkspaceError> {
		match self.entry(path)? {
			None => Ok(false),
			Some(EntryKind::File) => self.fs.remove_file(path).map(|()| true).map_err(io_error(path)),
			Some(EntryKind::Dir) => Err(WorkspaceError::NotAFile(path.clone())),
			Some(EntryKind::Symlink) => Err(WorkspaceError::Symlink(path.clone())),
		}
	}

	/// Removes a folder if it is empty; returns whether it was removed.
	pub(crate) fn remove_empty_dir(&self, path: &RelPath) -> Result<bool, WorkspaceError> {
		if self.entry(path)? != Some(EntryKind::Dir) || !self.fs.list(Some(path)).map_err(io_error(path))?.is_empty() {
			return Ok(false);
		}
		self.fs.remove_dir(path).map(|()| true).map_err(io_error(path))
	}

	/// Moves a file, creating the destination's folders.
	pub(crate) fn rename(&self, from: &RelPath, to: &RelPath) -> Result<(), WorkspaceError> {
		match self.entry(from)? {
			Some(EntryKind::File) => {}
			None => return Err(io_error(from)(ErrorKind::NotFound.into())),
			Some(EntryKind::Dir) => return Err(WorkspaceError::NotAFile(from.clone())),
			Some(EntryKind::Symlink) => return Err(WorkspaceError::Symlink(from.clone())),
		}
		self.prepare_file(to)?;
		self.fs.rename(from, to).map_err(io_error(from))
	}

	/// Every file in the project, skipping links, hidden folders and dependency or build output;
	/// stops at `limit` files.
	pub(crate) fn walk(&self, limit: usize) -> Result<Vec<RelPath>, WorkspaceError> {
		const SKIP: [&str; 8] = [
			"node_modules",
			"target",
			"dist",
			"build",
			"vendor",
			"out",
			"coverage",
			"__pycache__",
		];
		let mut files = Vec::new();
		let mut folders: Vec<Option<RelPath>> = vec![None];
		while let Some(folder) = folders.pop() {
			for name in self.list(folder.as_ref())? {
				let Ok(name_path) = RelPath::new(&name) else {
					continue;
				};
				let path = folder
					.as_ref()
					.map_or_else(|| name_path.clone(), |folder| folder.join(&name_path));
				match self.fs.entry(&path).map_err(io_error(&path))? {
					Some(EntryKind::Dir) if !name.starts_with('.') && !SKIP.contains(&name.as_str()) => {
						folders.push(Some(path));
					}
					Some(EntryKind::File) if files.len() < limit => files.push(path),
					_ => {}
				}
			}
		}
		files.sort();
		Ok(files)
	}

	/// Names in a folder, or in the root when `dir` is `None`; empty when the folder does not exist.
	pub(crate) fn list(&self, dir: Option<&RelPath>) -> Result<Vec<String>, WorkspaceError> {
		if let Some(dir) = dir {
			match self.entry(dir)? {
				None => return Ok(Vec::new()),
				Some(EntryKind::Dir) => {}
				Some(EntryKind::File) => return Err(WorkspaceError::NotADir(dir.clone())),
				Some(EntryKind::Symlink) => return Err(WorkspaceError::Symlink(dir.clone())),
			}
		}
		self.fs.list(dir).map_err(|source| WorkspaceError::Io {
			path: dir.map_or_else(|| ".".to_owned(), ToString::to_string),
			source,
		})
	}

	fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, WorkspaceError> {
		for ancestor in path.ancestors() {
			match self.fs.entry(&ancestor).map_err(io_error(&ancestor))? {
				None => return Ok(None),
				Some(EntryKind::Dir) => {}
				Some(EntryKind::File) => return Err(WorkspaceError::NotADir(ancestor)),
				Some(EntryKind::Symlink) => return Err(WorkspaceError::Symlink(ancestor)),
			}
		}
		self.fs.entry(path).map_err(io_error(path))
	}

	fn prepare_file(&self, path: &RelPath) -> Result<Vec<RelPath>, WorkspaceError> {
		match self.entry(path)? {
			Some(EntryKind::Dir) => return Err(WorkspaceError::NotAFile(path.clone())),
			Some(EntryKind::Symlink) => return Err(WorkspaceError::Symlink(path.clone())),
			Some(EntryKind::File) | None => {}
		}
		let mut created = Vec::new();
		for ancestor in path.ancestors() {
			if self.fs.entry(&ancestor).map_err(io_error(&ancestor))?.is_none() {
				self.fs.create_dir(&ancestor).map_err(io_error(&ancestor))?;
				created.push(ancestor);
			}
		}
		Ok(created)
	}
}

fn io_error(path: &RelPath) -> impl FnOnce(io::Error) -> WorkspaceError + '_ {
	move |source| WorkspaceError::Io {
		path: path.to_string(),
		source,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn workspace() -> (Arc<MemoryFs>, Workspace) {
		let fs = Arc::new(MemoryFs::new());
		let workspace = Workspace::new(fs.clone());
		(fs, workspace)
	}

	fn rel(path: &str) -> RelPath {
		RelPath::new(path).unwrap()
	}

	#[test]
	fn write_should_create_parents_and_report_them() {
		let (_fs, workspace) = workspace();

		let created = workspace.write(&rel("a/b/c.txt"), b"x").unwrap();

		assert_eq!(created, [rel("a"), rel("a/b")]);
	}

	#[test]
	fn read_should_return_none_for_missing_file() {
		let (_fs, workspace) = workspace();

		assert!(workspace.read(&rel("missing.txt")).unwrap().is_none());
	}

	#[test]
	fn write_should_fail_when_a_parent_is_a_file() {
		let (_fs, workspace) = workspace();
		workspace.write(&rel("a"), b"x").unwrap();

		assert!(matches!(
			workspace.write(&rel("a/b.txt"), b"x"),
			Err(WorkspaceError::NotADir(_))
		));
	}

	#[test]
	fn write_should_refuse_symlinked_parent() {
		let (fs, workspace) = workspace();
		fs.insert_symlink(&rel("link"));

		assert!(matches!(
			workspace.write(&rel("link/evil.txt"), b"x"),
			Err(WorkspaceError::Symlink(_))
		));
	}

	#[test]
	fn read_should_refuse_symlinked_file() {
		let (fs, workspace) = workspace();
		fs.insert_symlink(&rel("link.txt"));

		assert!(matches!(
			workspace.read(&rel("link.txt")),
			Err(WorkspaceError::Symlink(_))
		));
	}

	#[cfg(unix)]
	#[test]
	fn disk_workspace_should_refuse_symlinked_parent() {
		let dir = tempfile::tempdir().unwrap();
		let outside = tempfile::tempdir().unwrap();
		std::os::unix::fs::symlink(outside.path(), dir.path().join("link")).unwrap();
		let workspace = Workspace::open(dir.path()).unwrap();

		assert!(matches!(
			workspace.write(&rel("link/evil.txt"), b"x"),
			Err(WorkspaceError::Symlink(_))
		));
	}

	#[test]
	fn remove_empty_dir_should_keep_non_empty_folder() {
		let (_fs, workspace) = workspace();
		workspace.write(&rel("a/b.txt"), b"x").unwrap();

		assert!(!workspace.remove_empty_dir(&rel("a")).unwrap());
	}

	#[test]
	fn create_new_should_not_replace_existing_file() {
		let (fs, workspace) = workspace();
		workspace.write(&rel("busy"), b"1").unwrap();

		assert!(!workspace.create_new(&rel("busy"), b"2").unwrap());
		assert_eq!(fs.files()[&rel("busy")], b"1");
	}

	#[test]
	fn init_state_should_gitignore_the_state_dir() {
		let (fs, workspace) = workspace();
		workspace.init_state().unwrap();

		assert!(fs.files()[&Workspace::state_path(".gitignore")].contains(&b'*'));
	}
}
