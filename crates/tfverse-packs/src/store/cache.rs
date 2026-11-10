//! Downloaded packs, stored by content hash so pinned packs load offline.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tfverse_core::Pack;
use tfverse_core::brand;
use tfverse_core::model::Digest;

use crate::SourceError;

/// A folder of downloaded packs, one subfolder per content hash.
#[derive(Clone, Debug)]
pub struct Cache {
	dir: PathBuf,
}

impl Cache {
	/// A cache in `dir`.
	#[must_use]
	pub fn new(dir: impl Into<PathBuf>) -> Self {
		Self { dir: dir.into() }
	}

	/// The user's cache: `$TFVERSE_CACHE_DIR`, else the OS cache folder.
	#[must_use]
	pub fn user() -> Option<Self> {
		let env = |name: &str| {
			std::env::var_os(name)
				.filter(|value| !value.is_empty())
				.map(PathBuf::from)
		};
		let dir = env(&format!("{}CACHE_DIR", brand::ENV_PREFIX)).or_else(|| {
			let base = if cfg!(windows) {
				env("LOCALAPPDATA")
			} else if cfg!(target_os = "macos") {
				env("HOME").map(|home| home.join("Library/Caches"))
			} else {
				env("XDG_CACHE_HOME").or_else(|| env("HOME").map(|home| home.join(".cache")))
			};
			base.map(|base| base.join(brand::NAME))
		})?;
		Some(Self::new(dir.join("packs")))
	}

	/// The cached pack with `digest`, if present and intact.
	pub fn get(&self, digest: &Digest) -> Result<Option<Pack>, SourceError> {
		let dir = self.entry(digest);
		if !dir.is_dir() {
			return Ok(None);
		}
		let pack = Pack::load(&dir)?;
		Ok((pack.digest() == *digest).then_some(pack))
	}

	/// Stores `files`, the pack with `digest`; writes to a temporary folder first so readers never see half a pack.
	pub fn put(&self, digest: &Digest, files: &BTreeMap<String, Vec<u8>>) -> Result<(), SourceError> {
		let target = self.entry(digest);
		if target.is_dir() {
			return Ok(());
		}
		fs::create_dir_all(&self.dir).map_err(|source| io(&self.dir, source))?;
		let temp = self.dir.join(format!(".{}.{}", digest.hex(), std::process::id()));
		for (path, bytes) in files {
			let file = temp.join(path);
			if let Some(parent) = file.parent() {
				fs::create_dir_all(parent).map_err(|source| io(parent, source))?;
			}
			fs::write(&file, bytes).map_err(|source| io(&file, source))?;
		}
		match fs::rename(&temp, &target) {
			Ok(()) => Ok(()),
			Err(_) if target.is_dir() => fs::remove_dir_all(&temp).map_err(|source| io(&temp, source)),
			Err(source) => Err(io(&target, source)),
		}
	}

	fn entry(&self, digest: &Digest) -> PathBuf {
		self.dir.join(digest.hex())
	}
}

fn io(path: &Path, source: std::io::Error) -> SourceError {
	SourceError::Cache {
		path: path.to_path_buf(),
		source,
	}
}
