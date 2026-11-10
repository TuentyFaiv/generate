//! Content-addressed object cache in `.tfverse/objects`.

use super::{Workspace, WorkspaceError};
use crate::model::{Digest, RelPath};

/// Stores blobs by their SHA-256 digest.
#[derive(Clone, Debug)]
pub(crate) struct ObjectStore<'a> {
	workspace: &'a Workspace,
}

/// Why the object store failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError {
	/// Reading or writing the cache failed.
	#[error(transparent)]
	Workspace(#[from] WorkspaceError),
	/// A stored blob no longer matches its digest.
	#[error("cached object {0} is corrupt; delete `.tfverse/` to rebuild it")]
	Corrupt(Digest),
}

impl<'a> ObjectStore<'a> {
	pub(crate) fn new(workspace: &'a Workspace) -> Self {
		Self { workspace }
	}

	/// Stores `bytes`, returning their digest.
	pub(crate) fn put(&self, bytes: &[u8]) -> Result<Digest, StoreError> {
		let digest = Digest::of(bytes);
		let path = Self::path(&digest);
		if !self.workspace.exists(&path)? {
			self.workspace.init_state()?;
			self.workspace.write(&path, bytes)?;
		}
		Ok(digest)
	}

	/// Reads the blob for `digest`, verifying it.
	pub(crate) fn get(&self, digest: &Digest) -> Result<Option<Vec<u8>>, StoreError> {
		match self.workspace.read(&Self::path(digest))? {
			None => Ok(None),
			Some(bytes) if Digest::of(&bytes) == *digest => Ok(Some(bytes)),
			Some(_) => Err(StoreError::Corrupt(*digest)),
		}
	}

	fn path(digest: &Digest) -> RelPath {
		let hex = digest.hex();
		Workspace::state_path(&format!("objects/{}/{}", &hex[..2], &hex[2..]))
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;
	use crate::workspace::MemoryFs;

	fn workspace() -> (Arc<MemoryFs>, Workspace) {
		let fs = Arc::new(MemoryFs::new());
		(fs.clone(), Workspace::new(fs))
	}

	#[test]
	fn put_then_get_should_round_trip() {
		let (_fs, workspace) = workspace();
		let store = ObjectStore::new(&workspace);

		let digest = store.put(b"hello").unwrap();

		assert_eq!(store.get(&digest).unwrap().as_deref(), Some(&b"hello"[..]));
	}

	#[test]
	fn get_should_detect_corruption() {
		let (fs, workspace) = workspace();
		let store = ObjectStore::new(&workspace);
		let digest = store.put(b"hello").unwrap();
		fs.insert(&ObjectStore::path(&digest), b"tampered".to_vec());

		assert!(matches!(store.get(&digest), Err(StoreError::Corrupt(_))));
	}

	#[test]
	fn put_should_gitignore_the_state_dir() {
		let (fs, workspace) = workspace();
		ObjectStore::new(&workspace).put(b"x").unwrap();

		assert!(fs.files().contains_key(&Workspace::state_path(".gitignore")));
	}
}
