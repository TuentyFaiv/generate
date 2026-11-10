//! `status` and `why`: the state and provenance of generated files.

use semver::Version;
use serde::Serialize;

use super::{Engine, EngineError, PackProvider};
use crate::model::{Digest, Name, RelPath};
use crate::pack::{RenderRequest, render};
use crate::plan::origin_line;
use crate::workspace::Lockfile;

/// The state of one generated file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileStatus {
	/// Project-relative path.
	pub path: RelPath,
	/// Instance that generated it.
	pub instance: String,
	/// How it compares to what tfverse wrote.
	pub state: FileState,
}

/// How a generated file compares to what tfverse wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FileState {
	/// Exactly as generated.
	Clean,
	/// Edited since it was generated.
	Modified,
	/// Deleted since it was generated.
	Missing,
}

/// Where a line of a generated file comes from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Provenance {
	/// Instance that generated the file.
	pub instance: String,
	/// Pack name.
	pub pack: Name,
	/// Pack version.
	pub pack_version: Version,
	/// Piece name.
	pub piece: Name,
	/// Who wrote the line.
	pub line: LineOrigin,
}

/// Who wrote a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "origin", rename_all = "snake_case")]
#[non_exhaustive]
pub enum LineOrigin {
	/// Unchanged from the pack, at this 1-based line of the original.
	Pack {
		/// Line in the generated original.
		original_line: u32,
	},
	/// Added or changed since generation.
	User,
	/// The original is not available to compare against.
	Unknown,
}

impl Engine {
	/// The state of every generated file.
	pub fn status(&self) -> Result<Vec<FileStatus>, EngineError> {
		let lock = self.lockfile()?;
		let mut statuses = Vec::new();
		for instance in &lock.instances {
			for file in &instance.files {
				let state = match self.workspace.read(&file.path)? {
					None => FileState::Missing,
					Some(bytes) if Digest::of(&bytes) == file.base => FileState::Clean,
					Some(_) => FileState::Modified,
				};
				statuses.push(FileStatus {
					path: file.path.clone(),
					instance: instance.id.clone(),
					state,
				});
			}
		}
		statuses.sort_by(|a, b| a.path.cmp(&b.path));
		Ok(statuses)
	}

	/// Where 1-based `line` of `path` comes from; `None` when tfverse did not generate the file.
	pub fn why<P: PackProvider + ?Sized>(
		&self,
		path: &RelPath,
		line: u32,
		packs: &P,
	) -> Result<Option<Provenance>, EngineError> {
		let lock = self.lockfile()?;
		let Some((instance, _)) = lock.owner(path) else {
			return Ok(None);
		};
		let current = self.workspace.read(path)?.unwrap_or_default();
		let origin = match self.base_in(&lock, path, packs)? {
			None => LineOrigin::Unknown,
			Some(base) => match origin_line(
				&String::from_utf8_lossy(&base),
				&String::from_utf8_lossy(&current),
				line.saturating_sub(1),
			) {
				Some(original) => LineOrigin::Pack {
					original_line: original + 1,
				},
				None => LineOrigin::User,
			},
		};
		Ok(Some(Provenance {
			instance: instance.id.clone(),
			pack: instance.pack.clone(),
			pack_version: instance.pack_version.clone(),
			piece: instance.piece.clone(),
			line: origin,
		}))
	}

	/// The original generated content of `path`: from the cache, or rebuilt from its pack and verified.
	pub fn base<P: PackProvider + ?Sized>(&self, path: &RelPath, packs: &P) -> Result<Option<Vec<u8>>, EngineError> {
		self.base_in(&self.lockfile()?, path, packs)
	}

	fn base_in<P: PackProvider + ?Sized>(
		&self,
		lock: &Lockfile,
		path: &RelPath,
		packs: &P,
	) -> Result<Option<Vec<u8>>, EngineError> {
		let Some((instance, record)) = lock.owner(path) else {
			return Ok(None);
		};
		let store = self.store();
		if let Some(bytes) = store.get(&record.base)? {
			return Ok(Some(bytes));
		}
		let Some(piece) = packs
			.pack(&instance.pack, &instance.pack_version)
			.and_then(|pack| pack.piece(instance.piece.as_str()))
		else {
			return Ok(None);
		};
		let rendered = render(&RenderRequest {
			piece,
			values: &instance.inputs,
			stack: &instance.stack,
			variant: instance.variant.as_ref().map(Name::as_str),
			target: instance.target.as_ref(),
		})?;
		let Some(file) = rendered.files.into_iter().find(|file| file.path == *path) else {
			return Ok(None);
		};
		if Digest::of(&file.contents) != record.base {
			return Err(EngineError::BaseMismatch(path.clone()));
		}
		store.put(&file.contents)?;
		Ok(Some(file.contents))
	}
}
