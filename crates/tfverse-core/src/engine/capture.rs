//! Turning project code into a piece of a local pack, so it can be generated again under other names.

use std::fmt::Write as _;

use super::{Engine, EngineError};
use crate::model::{Glob, Name, PackManifest, Permissions, RelPath};
use crate::plan::{OnConflict, Plan, Rules, Validated};

const FIRST_VERSION: &str = "0.1.0";

/// A request to capture project files as a new piece.
#[derive(Clone, Copy, Debug)]
pub struct CaptureRequest<'a> {
	/// Folder of the pack to write into, e.g. `packs/mine`; created when missing.
	pub pack_dir: &'a RelPath,
	/// Pack name, used when that folder has no `pack.toml` yet.
	pub pack: &'a str,
	/// Name of the new piece.
	pub piece: &'a str,
	/// Project files to capture; the piece's target is the folder they share.
	pub files: &'a [RelPath],
	/// How the name is spelled in those files, e.g. `UserCard`; it becomes the piece's `name` input.
	pub example: Option<&'a str>,
	/// Allow replacing a piece that already exists.
	pub force: bool,
}

impl Engine {
	/// Plans writing `request.files` into a new piece of the pack at `request.pack_dir`; only that
	/// folder is written.
	pub fn plan_capture(&self, request: &CaptureRequest<'_>) -> Result<Plan<Validated>, EngineError> {
		let invalid = |message: String| EngineError::Capture(message);
		let piece = Name::new(request.piece).map_err(|error| invalid(error.to_string()))?;
		let pack = Name::new(request.pack).map_err(|error| invalid(error.to_string()))?;
		if request.files.is_empty() {
			return Err(invalid("no files to capture".to_owned()));
		}
		let target = common_folder(request.files).and_then(|folder| above_named(&folder, request.example));
		let piece_dir = request
			.pack_dir
			.join(&RelPath::new(&format!("pieces/{piece}")).map_err(|error| invalid(error.to_string()))?);
		let mut plan = Plan::new(format!("capture {piece} into {}", request.pack_dir));

		let mut found_example = false;
		for file in request.files {
			let bytes = self
				.workspace
				.read(file)?
				.ok_or_else(|| invalid(format!("`{file}` does not exist")))?;
			let text = String::from_utf8_lossy(&bytes);
			if text.contains("@tfverse") {
				return Err(invalid(format!(
					"`{file}` mentions `@tfverse`, which pack templates reserve for directives"
				)));
			}
			found_example |= request
				.example
				.is_some_and(|example| text.contains(example) || file.as_str().contains(&kebab(example)));
			let relative = match &target {
				Some(target) => file
					.as_str()
					.strip_prefix(&format!("{target}/"))
					.unwrap_or(file.as_str()),
				None => file.as_str(),
			};
			let path =
				piece_dir.join(&RelPath::new(&format!("files/{relative}")).map_err(|error| invalid(error.to_string()))?);
			plan.create(path, bytes);
		}
		if let Some(example) = request.example
			&& !found_example
		{
			plan.note(
				request.files[0].clone(),
				format!("`{example}` does not appear in these files, so the `name` input changes nothing"),
			);
		}

		let target_text = target.as_ref().map_or_else(String::new, ToString::to_string);
		let mut manifest = format!("schema_version = 1\nname = \"{piece}\"\ntarget = \"{target_text}\"\n");
		if let Some(example) = request.example {
			let example = example.replace(['"', '\\'], "");
			let _ = write!(
				manifest,
				"\n[inputs.name]\ntype = \"string\"\ndescription = \"Name of the new {piece}.\"\nexample = \"{example}\"\n"
			);
		}
		plan.create(piece_dir.join(&RelPath::constant("piece.toml")), manifest.into_bytes());

		let pack_toml = request.pack_dir.join(&RelPath::constant("pack.toml"));
		let write = format!(
			"{}**",
			target.as_ref().map_or_else(String::new, |target| format!("{target}/"))
		);
		match self.workspace.read(&pack_toml)? {
			None => plan.create(
				pack_toml,
				format!(
					"schema_version = 1\nname = \"{pack}\"\nversion = \"{FIRST_VERSION}\"\n\n[permissions]\nwrite = [\"{write}\"]\n"
				)
				.into_bytes(),
			),
			Some(bytes) => {
				let covered = toml::from_str::<PackManifest>(&String::from_utf8_lossy(&bytes))
					.is_ok_and(|manifest| request.files.iter().all(|file| manifest.permissions.allows_write(file)));
				if !covered {
					plan.note(
						pack_toml,
						format!("add `{write}` to `permissions.write` so the pack may write this piece's files"),
					);
				}
			}
		}

		let only_the_pack = Permissions {
			write: vec![Glob::new(&format!("{}/**", request.pack_dir)).map_err(|error| invalid(error.to_string()))?],
			..Permissions::default()
		};
		let rules = Rules {
			declared: &only_the_pack,
			granted: &only_the_pack,
			force: request.force,
			on_conflict: OnConflict::Refuse,
		};
		Ok(plan.validate(&self.workspace, &rules)?)
	}
}

/// The deepest folder every file is in, or `None` for the project root.
fn common_folder(files: &[RelPath]) -> Option<RelPath> {
	let folders: Vec<Vec<&str>> = files
		.iter()
		.map(|file| {
			let mut parts: Vec<&str> = file.as_str().split('/').collect();
			parts.pop();
			parts
		})
		.collect();
	let first = folders.first()?;
	let shared = (0..first.len())
		.take_while(|index| folders.iter().all(|parts| parts.get(*index) == first.get(*index)))
		.count();
	(shared > 0)
		.then(|| RelPath::new(&first[..shared].join("/")).ok())
		.flatten()
}

/// `folder` without its trailing folders named after `example`, so those folders get renamed with the piece.
fn above_named(folder: &RelPath, example: Option<&str>) -> Option<RelPath> {
	let squash = |text: &str| text.to_lowercase().replace(['-', '_', '.'], "");
	let mut parts: Vec<&str> = folder.as_str().split('/').collect();
	if let Some(example) = example.map(squash).filter(|example| !example.is_empty()) {
		while parts.last().is_some_and(|part| squash(part).contains(&example)) {
			parts.pop();
		}
	}
	(!parts.is_empty())
		.then(|| RelPath::new(&parts.join("/")).ok())
		.flatten()
}

/// `UserCard` → `user-card`, the spelling file names usually use.
fn kebab(text: &str) -> String {
	let mut output = String::with_capacity(text.len() + 4);
	for (index, c) in text.chars().enumerate() {
		if c.is_uppercase() && index > 0 {
			output.push('-');
		}
		output.extend(c.to_lowercase());
	}
	output
}

#[cfg(test)]
mod tests {
	use super::*;

	fn paths(paths: &[&str]) -> Vec<RelPath> {
		paths.iter().map(|path| RelPath::new(path).unwrap()).collect()
	}

	#[test]
	fn common_folder_should_be_the_deepest_shared_folder() {
		assert_eq!(
			common_folder(&paths(&["src/ui/card/card.tsx", "src/ui/card/card.test.tsx"])),
			Some(RelPath::new("src/ui/card").unwrap())
		);
		assert_eq!(
			common_folder(&paths(&["src/a.ts", "src/b/c.ts"])),
			Some(RelPath::new("src").unwrap())
		);
		assert_eq!(common_folder(&paths(&["a.ts", "src/b.ts"])), None);
	}

	#[test]
	fn above_named_should_lift_folders_named_after_the_example() {
		let folder = RelPath::new("src/ui/user-card").unwrap();

		assert_eq!(
			above_named(&folder, Some("UserCard")),
			Some(RelPath::new("src/ui").unwrap())
		);
		assert_eq!(above_named(&folder, None), Some(folder.clone()));
	}

	#[test]
	fn kebab_should_split_words() {
		assert_eq!(kebab("UserCard"), "user-card");
	}
}
