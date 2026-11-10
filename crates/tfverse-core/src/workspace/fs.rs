//! File access behind [`Fs`]: the real disk, or an in-memory tree for previews and tests.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::model::RelPath;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What a path is, without following symlinks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
	/// A regular file.
	File,
	/// A folder.
	Dir,
	/// A symlink, which tfverse never follows.
	Symlink,
}

/// Raw file operations below a project root; the engine adds the symlink and confinement checks.
pub trait Fs: Send + Sync + fmt::Debug {
	/// What `path` is, or `None` when nothing is there.
	fn entry(&self, path: &RelPath) -> io::Result<Option<EntryKind>>;
	/// Reads a file.
	fn read(&self, path: &RelPath) -> io::Result<Vec<u8>>;
	/// Creates or replaces a file atomically; its folder exists.
	fn write(&self, path: &RelPath, bytes: &[u8]) -> io::Result<()>;
	/// Creates a file, failing with [`ErrorKind::AlreadyExists`] if something is there.
	fn create_new(&self, path: &RelPath, bytes: &[u8]) -> io::Result<()>;
	/// Creates one folder; its parent exists.
	fn create_dir(&self, path: &RelPath) -> io::Result<()>;
	/// Removes a file or symlink.
	fn remove_file(&self, path: &RelPath) -> io::Result<()>;
	/// Removes an empty folder.
	fn remove_dir(&self, path: &RelPath) -> io::Result<()>;
	/// Moves a file.
	fn rename(&self, from: &RelPath, to: &RelPath) -> io::Result<()>;
	/// Sorted names in `dir`, or in the root when `dir` is `None`.
	fn list(&self, dir: Option<&RelPath>) -> io::Result<Vec<String>>;
}

/// A project folder on disk.
#[derive(Clone, Debug)]
pub struct DiskFs {
	root: PathBuf,
}

impl DiskFs {
	/// Opens the folder at `root`.
	pub fn open(root: &Path) -> io::Result<Self> {
		let root = root.canonicalize()?;
		if !root.is_dir() {
			return Err(io::Error::new(
				ErrorKind::NotADirectory,
				format!("`{}` is not a folder", root.display()),
			));
		}
		Ok(Self { root })
	}

	/// The canonical root.
	#[must_use]
	pub fn root(&self) -> &Path {
		&self.root
	}

	fn native(&self, path: &RelPath) -> PathBuf {
		path.under(&self.root)
	}
}

impl Fs for DiskFs {
	fn entry(&self, path: &RelPath) -> io::Result<Option<EntryKind>> {
		match fs::symlink_metadata(self.native(path)) {
			Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
			Err(error) => Err(error),
			Ok(meta) if meta.file_type().is_symlink() => Ok(Some(EntryKind::Symlink)),
			Ok(meta) if meta.is_dir() => Ok(Some(EntryKind::Dir)),
			Ok(_) => Ok(Some(EntryKind::File)),
		}
	}

	fn read(&self, path: &RelPath) -> io::Result<Vec<u8>> {
		fs::read(self.native(path))
	}

	fn write(&self, path: &RelPath, bytes: &[u8]) -> io::Result<()> {
		let target = self.native(path);
		let parent = target.parent().unwrap_or(&self.root);
		let temp = parent.join(format!(
			".{}.tfverse-{}-{}.tmp",
			path.file_name(),
			std::process::id(),
			TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
		));
		let result = (|| {
			let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
			file.write_all(bytes)?;
			file.sync_all()?;
			fs::rename(&temp, &target)
		})();
		if result.is_err() {
			let _ = fs::remove_file(&temp);
		}
		result
	}

	fn create_new(&self, path: &RelPath, bytes: &[u8]) -> io::Result<()> {
		OpenOptions::new()
			.write(true)
			.create_new(true)
			.open(self.native(path))?
			.write_all(bytes)
	}

	fn create_dir(&self, path: &RelPath) -> io::Result<()> {
		fs::create_dir(self.native(path))
	}

	fn remove_file(&self, path: &RelPath) -> io::Result<()> {
		fs::remove_file(self.native(path))
	}

	fn remove_dir(&self, path: &RelPath) -> io::Result<()> {
		fs::remove_dir(self.native(path))
	}

	fn rename(&self, from: &RelPath, to: &RelPath) -> io::Result<()> {
		fs::rename(self.native(from), self.native(to))
	}

	fn list(&self, dir: Option<&RelPath>) -> io::Result<Vec<String>> {
		let native = dir.map_or_else(|| self.root.clone(), |dir| self.native(dir));
		let mut names = Vec::new();
		for entry in fs::read_dir(native)? {
			if let Ok(name) = entry?.file_name().into_string() {
				names.push(name);
			}
		}
		names.sort();
		Ok(names)
	}
}

/// An in-memory project tree.
#[derive(Debug, Default)]
pub struct MemoryFs {
	nodes: Mutex<BTreeMap<RelPath, Node>>,
}

#[derive(Clone, Debug)]
enum Node {
	File(Vec<u8>),
	Dir,
	Symlink,
}

impl MemoryFs {
	/// An empty tree.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a file, creating its folders.
	pub fn insert(&self, path: &RelPath, bytes: impl Into<Vec<u8>>) {
		let mut nodes = self.nodes();
		for ancestor in path.ancestors() {
			nodes.insert(ancestor, Node::Dir);
		}
		nodes.insert(path.clone(), Node::File(bytes.into()));
	}

	/// Adds a symlink, to exercise symlink handling.
	pub fn insert_symlink(&self, path: &RelPath) {
		let mut nodes = self.nodes();
		for ancestor in path.ancestors() {
			nodes.insert(ancestor, Node::Dir);
		}
		nodes.insert(path.clone(), Node::Symlink);
	}

	/// Every file, by path.
	#[must_use]
	pub fn files(&self) -> BTreeMap<RelPath, Vec<u8>> {
		self
			.nodes()
			.iter()
			.filter_map(|(path, node)| match node {
				Node::File(bytes) => Some((path.clone(), bytes.clone())),
				Node::Dir | Node::Symlink => None,
			})
			.collect()
	}

	fn nodes(&self) -> MutexGuard<'_, BTreeMap<RelPath, Node>> {
		self.nodes.lock().unwrap_or_else(PoisonError::into_inner)
	}
}

fn require_parent(nodes: &BTreeMap<RelPath, Node>, path: &RelPath) -> io::Result<()> {
	match path.parent() {
		None => Ok(()),
		Some(parent) => match nodes.get(&parent) {
			Some(Node::Dir) => Ok(()),
			Some(_) => Err(ErrorKind::NotADirectory.into()),
			None => Err(ErrorKind::NotFound.into()),
		},
	}
}

fn is_child(dir: Option<&RelPath>, path: &RelPath) -> bool {
	path.parent().as_ref() == dir
}

impl Fs for MemoryFs {
	fn entry(&self, path: &RelPath) -> io::Result<Option<EntryKind>> {
		Ok(self.nodes().get(path).map(|node| match node {
			Node::File(_) => EntryKind::File,
			Node::Dir => EntryKind::Dir,
			Node::Symlink => EntryKind::Symlink,
		}))
	}

	fn read(&self, path: &RelPath) -> io::Result<Vec<u8>> {
		match self.nodes().get(path) {
			Some(Node::File(bytes)) => Ok(bytes.clone()),
			Some(Node::Dir) => Err(ErrorKind::IsADirectory.into()),
			Some(Node::Symlink) => Err(ErrorKind::InvalidInput.into()),
			None => Err(ErrorKind::NotFound.into()),
		}
	}

	fn write(&self, path: &RelPath, bytes: &[u8]) -> io::Result<()> {
		let mut nodes = self.nodes();
		require_parent(&nodes, path)?;
		if matches!(nodes.get(path), Some(Node::Dir)) {
			return Err(ErrorKind::IsADirectory.into());
		}
		nodes.insert(path.clone(), Node::File(bytes.to_vec()));
		Ok(())
	}

	fn create_new(&self, path: &RelPath, bytes: &[u8]) -> io::Result<()> {
		let mut nodes = self.nodes();
		require_parent(&nodes, path)?;
		if nodes.contains_key(path) {
			return Err(ErrorKind::AlreadyExists.into());
		}
		nodes.insert(path.clone(), Node::File(bytes.to_vec()));
		Ok(())
	}

	fn create_dir(&self, path: &RelPath) -> io::Result<()> {
		let mut nodes = self.nodes();
		require_parent(&nodes, path)?;
		if nodes.contains_key(path) {
			return Err(ErrorKind::AlreadyExists.into());
		}
		nodes.insert(path.clone(), Node::Dir);
		Ok(())
	}

	fn remove_file(&self, path: &RelPath) -> io::Result<()> {
		let mut nodes = self.nodes();
		match nodes.get(path) {
			Some(Node::File(_) | Node::Symlink) => {
				nodes.remove(path);
				Ok(())
			}
			Some(Node::Dir) => Err(ErrorKind::IsADirectory.into()),
			None => Err(ErrorKind::NotFound.into()),
		}
	}

	fn remove_dir(&self, path: &RelPath) -> io::Result<()> {
		let mut nodes = self.nodes();
		match nodes.get(path) {
			Some(Node::Dir) if nodes.keys().any(|other| is_child(Some(path), other)) => {
				Err(ErrorKind::DirectoryNotEmpty.into())
			}
			Some(Node::Dir) => {
				nodes.remove(path);
				Ok(())
			}
			Some(_) => Err(ErrorKind::NotADirectory.into()),
			None => Err(ErrorKind::NotFound.into()),
		}
	}

	fn rename(&self, from: &RelPath, to: &RelPath) -> io::Result<()> {
		let mut nodes = self.nodes();
		require_parent(&nodes, to)?;
		match nodes.get(from) {
			Some(Node::File(_) | Node::Symlink) => {
				let node = nodes.remove(from).ok_or(ErrorKind::NotFound)?;
				nodes.insert(to.clone(), node);
				Ok(())
			}
			Some(Node::Dir) => Err(ErrorKind::IsADirectory.into()),
			None => Err(ErrorKind::NotFound.into()),
		}
	}

	fn list(&self, dir: Option<&RelPath>) -> io::Result<Vec<String>> {
		let nodes = self.nodes();
		if let Some(dir) = dir {
			match nodes.get(dir) {
				Some(Node::Dir) => {}
				Some(_) => return Err(ErrorKind::NotADirectory.into()),
				None => return Err(ErrorKind::NotFound.into()),
			}
		}
		Ok(
			nodes
				.keys()
				.filter(|path| is_child(dir, path))
				.map(|path| path.file_name().to_owned())
				.collect(),
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn rel(path: &str) -> RelPath {
		RelPath::new(path).unwrap()
	}

	fn round_trip(fs: &dyn Fs) {
		fs.create_dir(&rel("a")).unwrap();
		fs.write(&rel("a/b.txt"), b"one").unwrap();
		fs.write(&rel("a/b.txt"), b"two").unwrap();
		fs.create_new(&rel("a/c.txt"), b"x").unwrap();

		assert_eq!(fs.read(&rel("a/b.txt")).unwrap(), b"two");
		assert_eq!(fs.list(Some(&rel("a"))).unwrap(), ["b.txt", "c.txt"]);
		assert_eq!(fs.list(None).unwrap(), ["a"]);
		assert_eq!(
			fs.create_new(&rel("a/c.txt"), b"y").unwrap_err().kind(),
			ErrorKind::AlreadyExists
		);
		assert!(fs.remove_dir(&rel("a")).is_err());

		fs.rename(&rel("a/c.txt"), &rel("a/d.txt")).unwrap();
		fs.remove_file(&rel("a/b.txt")).unwrap();
		fs.remove_file(&rel("a/d.txt")).unwrap();
		fs.remove_dir(&rel("a")).unwrap();

		assert_eq!(fs.entry(&rel("a")).unwrap(), None);
	}

	#[test]
	fn disk_fs_should_follow_the_contract() {
		let dir = tempfile::tempdir().unwrap();

		round_trip(&DiskFs::open(dir.path()).unwrap());
	}

	#[test]
	fn memory_fs_should_follow_the_contract() {
		round_trip(&MemoryFs::new());
	}

	#[test]
	fn memory_fs_should_require_parent_folder() {
		let fs = MemoryFs::new();

		assert_eq!(fs.write(&rel("a/b.txt"), b"x").unwrap_err().kind(), ErrorKind::NotFound);
	}

	#[test]
	fn disk_fs_write_should_leave_no_temp_files() {
		let dir = tempfile::tempdir().unwrap();
		let fs = DiskFs::open(dir.path()).unwrap();
		fs.write(&rel("a.txt"), b"x").unwrap();

		assert_eq!(fs.list(None).unwrap(), ["a.txt"]);
	}

	#[cfg(unix)]
	#[test]
	fn disk_fs_should_report_symlinks_without_following() {
		let dir = tempfile::tempdir().unwrap();
		std::os::unix::fs::symlink("/etc/hosts", dir.path().join("link")).unwrap();

		assert_eq!(
			DiskFs::open(dir.path()).unwrap().entry(&rel("link")).unwrap(),
			Some(EntryKind::Symlink)
		);
	}
}
