//! `pack.toml` and `piece.toml`.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::Glob;

/// The only manifest schema version this build reads.
pub const SCHEMA_VERSION: u32 = 1;

/// A pack, piece or variant name: lowercase letters, digits and `-`, starting with a letter.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(String);

/// Why a name was rejected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("`{0}` must be 1-64 lowercase letters, digits or `-`, starting with a letter")]
pub struct NameError(String);

impl Name {
	/// Validates `name`.
	pub fn new(name: &str) -> Result<Self, NameError> {
		let valid = (1..=64).contains(&name.len())
			&& name.starts_with(|c: char| c.is_ascii_lowercase())
			&& name
				.chars()
				.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
			&& !name.ends_with('-')
			&& !name.contains("--");
		if valid {
			Ok(Self(name.to_owned()))
		} else {
			Err(NameError(name.to_owned()))
		}
	}

	/// The name as written.
	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl fmt::Display for Name {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl fmt::Debug for Name {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.0, f)
	}
}

impl Serialize for Name {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.0)
	}
}

impl<'de> Deserialize<'de> for Name {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let text = Cow::<str>::deserialize(deserializer)?;
		Self::new(&text).map_err(serde::de::Error::custom)
	}
}

impl JsonSchema for Name {
	fn schema_name() -> Cow<'static, str> {
		"Name".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({
			"type": "string",
			"pattern": "^[a-z](?:[a-z0-9]|-(?=[a-z0-9])){0,63}$"
		})
	}
}

/// `pack.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
	/// Manifest format version; must be `1`.
	pub schema_version: u32,
	/// Pack name.
	pub name: Name,
	/// Semantic version of the pack.
	pub version: Version,
	/// One-line summary.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	/// SPDX license expression.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub license: Option<String>,
	/// What the pack may do in a project.
	#[serde(default)]
	pub permissions: Permissions,
	/// Changes to project code that updates crossing a version make.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub migrations: Vec<Migration>,
}

/// A change to project code made when an update crosses `to`, shown in the plan like any other change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Migration {
	/// Runs when updating from a version below this one to this one or later.
	pub to: Version,
	/// What it changes, for the plan.
	pub description: String,
	/// Project files it runs on; they must also be in `permissions.write`.
	pub files: Vec<Glob>,
	/// Renames an identifier, matching whole words only.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub rename: Option<Rename>,
	/// An ast-grep rule file in the pack, e.g. `migrations/2.0.0.yml`; needs the `ast-grep` program.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub ast_grep: Option<String>,
}

/// An identifier rename.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rename {
	/// The old name.
	pub from: String,
	/// The new name.
	pub to: String,
}

/// What a pack may do; granted by the user on install.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
	/// Paths the pack may create or modify.
	#[serde(default)]
	pub write: Vec<Glob>,
	/// External commands the pack may run.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub commands: Vec<String>,
	/// Whether the pack may add dependencies.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub dependencies: bool,
}

/// `piece.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PieceManifest {
	/// Manifest format version; must be `1`.
	pub schema_version: u32,
	/// Piece name; must match its folder.
	pub name: Name,
	/// What the piece is for.
	#[serde(default)]
	pub kind: PieceKind,
	/// One-line summary.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	/// Default output folder, relative to the project root; may contain examples. Empty for the root.
	pub target: String,
	/// Inputs by name.
	#[serde(default)]
	pub inputs: BTreeMap<String, Input>,
	/// Stack-specific template sets, tried in order.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub variants: Vec<Variant>,
	/// Conditions for individual template files.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub files: Vec<FileRule>,
	/// Edits to existing project files.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub patches: Vec<PatchSpec>,
	/// Pieces of the same pack this one builds on; added first when the project lacks them.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub requires: Vec<Name>,
	/// Packages the generated code imports.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub dependencies: Vec<Dependency>,
}

/// What a piece is for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PieceKind {
	/// A code module such as a component, model or middleware.
	#[default]
	Piece,
	/// A saved set of answers for other pieces.
	Preset,
	/// Project configuration such as lint or CI settings.
	Config,
}

/// One input of a piece.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
	/// Text; with a multi-word `example`, an identifier replaced in every case form.
	String {
		/// Shown in help and schemas.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		description: Option<String>,
		/// Interactive prompt.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		prompt: Option<String>,
		/// Text in the templates that this input replaces.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		example: Option<String>,
		/// Value used when none is given.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		default: Option<String>,
	},
	/// `true` or `false`, for conditions.
	Bool {
		/// Shown in help and schemas.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		description: Option<String>,
		/// Interactive prompt.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		prompt: Option<String>,
		/// Value used when none is given.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		default: Option<bool>,
	},
	/// One of a fixed set of choices, for conditions.
	Enum {
		/// Shown in help and schemas.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		description: Option<String>,
		/// Interactive prompt.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		prompt: Option<String>,
		/// Allowed values.
		choices: Vec<String>,
		/// Value used when none is given.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		default: Option<String>,
	},
	/// A whole number.
	Number {
		/// Shown in help and schemas.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		description: Option<String>,
		/// Interactive prompt.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		prompt: Option<String>,
		/// Number in the templates that this input replaces.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		example: Option<i64>,
		/// Value used when none is given.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		default: Option<i64>,
		/// Smallest allowed value.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		min: Option<i64>,
		/// Largest allowed value.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		max: Option<i64>,
	},
}

impl Input {
	/// Help text.
	#[must_use]
	pub fn description(&self) -> Option<&str> {
		match self {
			Self::String { description, .. }
			| Self::Bool { description, .. }
			| Self::Enum { description, .. }
			| Self::Number { description, .. } => description.as_deref(),
		}
	}
}

/// A set of templates for one stack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Variant {
	/// Variant name.
	pub name: Name,
	/// Project facts that must all hold, e.g. `{ framework = "react" }`.
	#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
	pub when: BTreeMap<Fact, String>,
	/// Template folder inside the piece; defaults to `files/<name>`.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub files: Option<String>,
	/// Output folder instead of the piece's `target`, e.g. `lib/ui` for Flutter.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub target: Option<String>,
}

/// A kind of project fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Fact {
	/// e.g. `react`, `svelte`, `express`.
	Framework,
	/// e.g. `typescript`, `rust`.
	Language,
	/// e.g. `node`, `bun`, `deno`, `tokio`.
	Runtime,
	/// The library whose conventions database code follows: ORMs, ODMs and query builders,
	/// e.g. `prisma`, `diesel`, `activerecord`. Drivers, SQL and schema files are not part of it.
	Database,
	/// e.g. `tailwind`, `stylex`.
	Styling,
	/// e.g. `bun`, `pnpm`, `cargo`.
	PackageManager,
}

impl Fact {
	/// The fact's key in conditions and config.
	#[must_use]
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Framework => "framework",
			Self::Language => "language",
			Self::Runtime => "runtime",
			Self::Database => "database",
			Self::Styling => "styling",
			Self::PackageManager => "package_manager",
		}
	}
}

/// A package the generated code imports.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
	/// Package registry.
	pub ecosystem: Ecosystem,
	/// Package name, e.g. `zod` or `github.com/google/uuid`.
	pub name: String,
	/// Version requirement in the ecosystem's syntax, e.g. `^4.1`.
	pub version: String,
	/// Cargo features or Python extras, e.g. `["derive"]`.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub features: Vec<String>,
	/// Only needed to develop and test.
	#[serde(default, skip_serializing_if = "std::ops::Not::not")]
	pub dev: bool,
	/// Variants that need it; empty for every variant.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub variants: Vec<Name>,
	/// Condition on inputs or stack facts.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub when: Option<String>,
}

/// A package registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Ecosystem {
	/// npm, also used by Bun, pnpm, Yarn and Deno.
	Npm,
	/// crates.io.
	Cargo,
	/// The Python Package Index.
	Pypi,
	/// Go modules.
	Go,
	/// pub.dev.
	Pub,
	/// The Ruby gem registry.
	Rubygems,
	/// Packagist.
	Packagist,
}

impl Ecosystem {
	/// The project file that lists these dependencies.
	#[must_use]
	pub fn manifest(self) -> &'static str {
		match self {
			Self::Npm => "package.json",
			Self::Cargo => "Cargo.toml",
			Self::Pypi => "pyproject.toml",
			Self::Go => "go.mod",
			Self::Pub => "pubspec.yaml",
			Self::Rubygems => "Gemfile",
			Self::Packagist => "composer.json",
		}
	}
}

/// A condition on one template file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileRule {
	/// Template path inside the variant folder.
	pub path: String,
	/// Condition that keeps the file.
	pub when: String,
}

/// An edit to an existing project file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PatchSpec {
	/// Inserts text next to an anchor line; works in every language.
	Anchor {
		/// Project file to edit; may contain examples.
		file: String,
		/// Text identifying the anchor line.
		anchor: String,
		/// Where to insert relative to the anchor line.
		#[serde(default)]
		position: Position,
		/// Text to insert; may contain examples.
		insert: String,
		/// Condition that keeps the patch.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		when: Option<String>,
	},
	/// Adds a JavaScript or TypeScript import after the existing imports.
	Import {
		/// Project file to edit; may contain examples.
		file: String,
		/// The import statement; may contain examples.
		import: String,
		/// Condition that keeps the patch.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		when: Option<String>,
	},
	/// Adds CSS custom properties to a block such as `:root`, keeping values the project already set.
	CssVars {
		/// Project CSS file to edit; may contain examples.
		file: String,
		/// The block's selector or at-rule, e.g. `:root`, `.dark` or `@theme inline`.
		selector: String,
		/// Properties without the leading `--`, e.g. `sidebar-border = "220 13% 91%"`.
		vars: BTreeMap<String, String>,
		/// Condition that keeps the patch.
		#[serde(default, skip_serializing_if = "Option::is_none")]
		when: Option<String>,
	},
}

impl PatchSpec {
	/// The project file the patch edits.
	#[must_use]
	pub fn file(&self) -> &str {
		match self {
			Self::Anchor { file, .. } | Self::Import { file, .. } | Self::CssVars { file, .. } => file,
		}
	}

	/// The condition that keeps the patch.
	#[must_use]
	pub fn when(&self) -> Option<&str> {
		match self {
			Self::Anchor { when, .. } | Self::Import { when, .. } | Self::CssVars { when, .. } => when.as_deref(),
		}
	}
}

/// Where an anchor patch inserts its text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Position {
	/// On the lines after the anchor.
	#[default]
	After,
	/// On the lines before the anchor.
	Before,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn name_should_accept_kebab_case() {
		assert!(Name::new("react-component2").is_ok());
	}

	#[test]
	fn name_should_reject_uppercase() {
		assert!(Name::new("Component").is_err());
	}

	#[test]
	fn name_should_reject_double_dash() {
		assert!(Name::new("a--b").is_err());
	}

	#[test]
	fn piece_manifest_should_parse_tagged_inputs() {
		let manifest: PieceManifest = toml::from_str(
			r#"
			schema_version = 1
			name = "component"
			target = "src/components"

			[inputs.name]
			type = "string"
			example = "MyExample"

			[inputs.styled]
			type = "bool"
			default = false
			"#,
		)
		.unwrap();

		assert!(matches!(
			manifest.inputs.get("styled"),
			Some(Input::Bool {
				default: Some(false),
				..
			})
		));
	}

	#[test]
	fn piece_manifest_should_reject_unknown_fields() {
		let result =
			toml::from_str::<PieceManifest>("schema_version = 1\nname = \"x\"\ntarget = \"src\"\nunknown = true\n");

		assert!(result.is_err());
	}
}
