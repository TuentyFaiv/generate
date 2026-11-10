//! The engine: one entry point shared by the CLI, the MCP server and the GUI.

mod builder;
mod capture;
mod planner;
mod status;
mod update;

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use semver::Version;

pub use builder::EngineBuilder;
pub use capture::CaptureRequest;
pub use status::{FileState, FileStatus, LineOrigin, Provenance};
pub use update::UpdateRequest;

use crate::event::EventSink;
use crate::history::{self, Edit, History, HistoryError, Transaction};
use crate::migrate::AstGrep;
use crate::model::{Name, Permissions, RelPath, Values};
use crate::pack::{InputError, Pack, RenderError};
use crate::patch::{PatchError, PatchKind, Patchers};
use crate::plan::{Action, Plan, Rejected, Validated};
use crate::workspace::{
	ConfigError, LockError, Lockfile, ObjectStore, ProjectConfig, ProjectModel, StoreError, Workspace, WorkspaceError,
};

/// Finds packs by name and version, e.g. to rebuild an original that is not cached.
pub trait PackProvider {
	/// The pack `name` at exactly `version`.
	fn pack(&self, name: &Name, version: &Version) -> Option<&Pack>;
}

impl PackProvider for [Pack] {
	fn pack(&self, name: &Name, version: &Version) -> Option<&Pack> {
		self
			.iter()
			.find(|pack| pack.manifest().name == *name && pack.manifest().version == *version)
	}
}

/// A request to generate one piece.
#[derive(Clone, Copy, Debug)]
pub struct PieceRequest<'a> {
	/// The pack.
	pub pack: &'a Pack,
	/// Piece name.
	pub piece: &'a str,
	/// Input values; config defaults fill the rest.
	pub inputs: &'a Values,
	/// Variant to use instead of detection.
	pub variant: Option<&'a str>,
	/// Output folder instead of config or the piece's target.
	pub target: Option<&'a RelPath>,
	/// Permissions the user granted; defaults to what the pack declares.
	pub granted: Option<&'a Permissions>,
	/// Allow replacing existing files that differ.
	pub force: bool,
}

/// Why an engine operation failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EngineError {
	/// The pack has no such piece.
	#[error("pack `{pack}` has no piece `{piece}`")]
	UnknownPiece {
		/// Pack name.
		pack: String,
		/// Requested piece.
		piece: String,
	},
	/// A file to patch does not exist.
	#[error("cannot patch `{0}`: the file does not exist")]
	PatchTarget(RelPath),
	/// A patch failed.
	#[error("patch on `{file}`: {source}")]
	Patch {
		/// File being patched.
		file: RelPath,
		/// The cause.
		source: PatchError,
	},
	/// A configured patcher is invalid.
	#[error("patcher `{kind}`: {source}")]
	Patcher {
		/// Patch kind.
		kind: PatchKind,
		/// The cause.
		source: PatchError,
	},
	/// The pack's files differ from the ones the lockfile pinned for this version.
	#[error("pack `{name}` {version} differs from the version in the lockfile; it was changed without a new version")]
	PackChanged {
		/// Pack name.
		name: Name,
		/// Pack version.
		version: Version,
	},
	/// Project files cannot be captured as a piece.
	#[error("capture: {0}")]
	Capture(String),
	/// The project has nothing generated from the pack.
	#[error("this project has no pieces from pack `{0}` to update")]
	NotInstalled(Name),
	/// The pack is not signed by the key that signed the version in the lockfile.
	#[error("pack `{name}` was signed by key {expected}, but this version is {}", .actual.as_deref().map_or_else(|| "not signed by a trusted key".to_owned(), |key| format!("signed by key {key}")))]
	SignerChanged {
		/// Pack name.
		name: Name,
		/// Key id in the lockfile.
		expected: String,
		/// Key id of this version's verified signature.
		actual: Option<String>,
	},
	/// Two pieces in one plan write the same file.
	#[error("two pieces in this plan write `{0}`")]
	Overlap(RelPath),
	/// An original could not be rebuilt to the recorded hash.
	#[error("rebuilt original of `{0}` does not match the lockfile")]
	BaseMismatch(RelPath),
	/// Invalid inputs.
	#[error(transparent)]
	Input(#[from] InputError),
	/// Rendering failed.
	#[error(transparent)]
	Render(#[from] RenderError),
	/// The plan was rejected.
	#[error(transparent)]
	Rejected(#[from] Rejected),
	/// Config is invalid.
	#[error(transparent)]
	Config(#[from] ConfigError),
	/// The lockfile is invalid.
	#[error(transparent)]
	Lock(#[from] LockError),
	/// A transaction failed.
	#[error(transparent)]
	History(#[from] HistoryError),
	/// The object cache failed.
	#[error(transparent)]
	Store(#[from] StoreError),
	/// A project file operation failed.
	#[error(transparent)]
	Workspace(#[from] WorkspaceError),
}

/// The engine for one project; `Send + Sync`, so a GUI or server can keep and share it.
pub struct Engine {
	workspace: Workspace,
	config: ProjectConfig,
	patchers: Patchers,
	ast_grep: Arc<dyn AstGrep>,
	sink: Arc<dyn EventSink>,
}

impl fmt::Debug for Engine {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Engine")
			.field("workspace", &self.workspace)
			.field("patchers", &self.patchers)
			.finish_non_exhaustive()
	}
}

impl Engine {
	/// Configures an engine.
	pub fn builder() -> EngineBuilder {
		EngineBuilder::default()
	}

	/// Opens the project at `root` with the user's own config and no event sink.
	pub fn open(root: &Path) -> Result<Self, EngineError> {
		Self::builder().open(root)
	}

	/// The project config, read when the engine opened.
	#[must_use]
	pub fn config(&self) -> &ProjectConfig {
		&self.config
	}

	/// Detects the project model.
	pub fn project(&self) -> Result<ProjectModel, EngineError> {
		Ok(crate::workspace::detect(&self.workspace, &self.config)?)
	}

	/// The lockfile, or an empty one.
	pub fn lockfile(&self) -> Result<Lockfile, EngineError> {
		Ok(Lockfile::load(&self.workspace)?)
	}

	/// Applies a validated plan as one transaction.
	pub fn apply(&self, plan: Plan<Validated>) -> Result<Transaction, EngineError> {
		let (summary, changes, record) = plan.into_parts();
		let lock_after = if record.instances.is_empty() {
			None
		} else {
			let mut lock = self.lockfile()?;
			if let Some(pack) = record.pack {
				lock.upsert_pack(pack);
			}
			for instance in record.instances {
				lock.upsert(instance);
			}
			Some(lock.to_toml())
		};
		let edits: Vec<Edit<'_>> = changes
			.iter()
			.filter(|change| change.action != Action::Unchanged)
			.map(|change| Edit {
				path: &change.path,
				expected: change.current,
				contents: (change.action != Action::Delete).then_some(change.contents.as_slice()),
			})
			.collect();
		let transaction = history::apply(
			&self.workspace,
			summary,
			&edits,
			lock_after.as_deref().map(str::as_bytes),
			self.sink.as_ref(),
		)?;
		let store = self.store();
		for blob in record.blobs {
			store.put(&blob)?;
		}
		Ok(transaction)
	}

	/// Undoes the last transaction.
	pub fn undo(&self, force: bool) -> Result<Option<Transaction>, EngineError> {
		Ok(history::undo(&self.workspace, force, self.sink.as_ref())?)
	}

	/// Transactions that can be undone, oldest first.
	pub fn history(&self) -> Result<Vec<Transaction>, EngineError> {
		Ok(History::new(&self.workspace).entries()?)
	}

	fn store(&self) -> ObjectStore<'_> {
		ObjectStore::new(&self.workspace)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn engine_should_be_send_and_sync() {
		fn assert_send_sync<T: Send + Sync>() {}
		assert_send_sync::<Engine>();
	}
}
