//! Packs: loading and validating them, resolving inputs and rendering pieces.

mod inputs;
mod render;

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub use inputs::InputError;
pub(crate) use inputs::resolve as resolve_inputs;
pub use render::RenderError;
pub(crate) use render::{RenderRequest, Rendered, RenderedFile, render};

use crate::model::{
	Dependency, Digest, Input, Name, PackManifest, PieceManifest, RelPath, SCHEMA_VERSION, Value, Variant, words,
};
use crate::template::{self, Example, Expr, Replacer};

const PACK_MANIFEST: &str = "pack.toml";
/// The pack's minisign signature, kept out of its digest.
pub const SIGNATURE_FILE: &str = "pack.minisig";
const MAX_SIGNATURE_SIZE: usize = 4096;
const MIGRATIONS_DIR: &str = "migrations/";
const MAX_RULE_SIZE: usize = 64 * 1024;
const PIECE_MANIFEST: &str = "piece.toml";
const PIECES_DIR: &str = "pieces";
const DEFAULT_FILES_DIR: &str = "files";
const MAX_FILES: usize = 4096;
const MAX_FILE_SIZE: u64 = 1024 * 1024;
const MAX_PACK_SIZE: u64 = 64 * 1024 * 1024;
const STACK_PREFIX: &str = "stack.";

/// A loaded, validated pack.
#[derive(Clone, Debug)]
pub struct Pack {
	manifest: PackManifest,
	pieces: BTreeMap<Name, Piece>,
	digest: Digest,
	source: Option<String>,
	signature: Option<String>,
	signer: Option<String>,
	rules: BTreeMap<String, String>,
}

/// A loaded, validated piece.
#[derive(Clone, Debug)]
pub struct Piece {
	manifest: PieceManifest,
	templates: BTreeMap<String, Vec<u8>>,
}

/// Why a pack could not be loaded.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PackError {
	/// Reading a file failed.
	#[error("could not read `{path}`: {source}")]
	Io {
		/// The file.
		path: PathBuf,
		/// The cause.
		source: std::io::Error,
	},
	/// Packs may not contain symlinks.
	#[error("`{0}` is a symlink; packs may not contain symlinks")]
	Symlink(PathBuf),
	/// Too many or too large files.
	#[error("pack exceeds a limit: {0}")]
	Limit(String),
	/// A file path that is not portable or escapes the pack.
	#[error("invalid file path `{0}` in pack")]
	BadPath(String),
	/// `pack.toml` is missing.
	#[error("`{PACK_MANIFEST}` not found")]
	MissingManifest,
	/// A manifest is not valid TOML for its schema.
	#[error("`{file}`: {message}")]
	Parse {
		/// Manifest path inside the pack.
		file: String,
		/// Parser message.
		message: String,
	},
	/// A manifest uses a schema version this build does not read.
	#[error("`{file}` uses schema_version {version}; this tfverse reads version {SCHEMA_VERSION}")]
	UnsupportedSchema {
		/// Manifest path inside the pack.
		file: String,
		/// Version found.
		version: u32,
	},
	/// A manifest or template is inconsistent.
	#[error("`{file}`: {message}")]
	Invalid {
		/// File inside the pack.
		file: String,
		/// What is wrong.
		message: String,
	},
}

impl Pack {
	/// Loads a pack folder from disk.
	pub fn load(dir: &Path) -> Result<Self, PackError> {
		let mut files = BTreeMap::new();
		let mut total = 0;
		read_dir(dir, "", &mut files, &mut total)?;
		Self::from_files(files)
	}

	/// Builds a pack from its files, keyed by `/`-separated paths relative to the pack root.
	pub fn from_files(mut files: BTreeMap<String, Vec<u8>>) -> Result<Self, PackError> {
		if files.len() > MAX_FILES {
			return Err(PackError::Limit(format!("more than {MAX_FILES} files")));
		}
		for path in files.keys() {
			RelPath::new(path).map_err(|_| PackError::BadPath(path.clone()))?;
		}

		let manifest: PackManifest = parse(
			PACK_MANIFEST,
			files.get(PACK_MANIFEST).ok_or(PackError::MissingManifest)?,
		)?;
		check_schema(PACK_MANIFEST, manifest.schema_version)?;
		let signature = files
			.remove(SIGNATURE_FILE)
			.map(|bytes| {
				if bytes.len() > MAX_SIGNATURE_SIZE {
					return Err(invalid(SIGNATURE_FILE, "too large for a signature"));
				}
				String::from_utf8(bytes).map_err(|_| invalid(SIGNATURE_FILE, "not UTF-8 text"))
			})
			.transpose()?;
		let digest = Digest::of_files(files.iter().map(|(path, bytes)| (path.as_str(), bytes.as_slice())));

		let rules = migration_rules(&manifest, &files)?;
		let mut grouped: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
		for (path, bytes) in files {
			if let Some((piece, rest)) = path.strip_prefix("pieces/").and_then(|rest| rest.split_once('/')) {
				grouped
					.entry(piece.to_owned())
					.or_default()
					.insert(rest.to_owned(), bytes);
			}
		}

		let mut pieces = BTreeMap::new();
		for (folder, mut piece_files) in grouped {
			let manifest_path = format!("{PIECES_DIR}/{folder}/{PIECE_MANIFEST}");
			let bytes = piece_files
				.remove(PIECE_MANIFEST)
				.ok_or_else(|| invalid(&manifest_path, "missing piece manifest"))?;
			let piece = Piece::new(&manifest_path, &folder, parse(&manifest_path, &bytes)?, piece_files)?;
			pieces.insert(piece.manifest.name.clone(), piece);
		}
		let pack = Self {
			manifest,
			pieces,
			digest,
			source: None,
			signature,
			signer: None,
			rules,
		};
		pack.validate_links()?;
		Ok(pack)
	}

	/// The same pack, remembering where it came from, e.g. `gh:acme/packs@v1.2.0`.
	#[must_use]
	pub fn with_source(mut self, source: impl Into<String>) -> Self {
		self.source = Some(source.into());
		self
	}

	/// Where the pack came from, if known.
	#[must_use]
	pub fn source(&self) -> Option<&str> {
		self.source.as_deref()
	}

	/// SHA-256 of every file in the pack except its signature.
	#[must_use]
	pub fn digest(&self) -> Digest {
		self.digest
	}

	/// The ast-grep rule at `path` inside the pack, e.g. `migrations/2.0.0.yml`.
	#[must_use]
	pub fn migration_rule(&self, path: &str) -> Option<&str> {
		self.rules.get(path).map(String::as_str)
	}

	/// The contents of `pack.minisig`, if the pack is signed.
	#[must_use]
	pub fn signature(&self) -> Option<&str> {
		self.signature.as_deref()
	}

	/// What a signature covers: the name, version and digest, one line.
	#[must_use]
	pub fn signed_message(&self) -> String {
		format!(
			"tfverse pack {} {} {}\n",
			self.manifest.name, self.manifest.version, self.digest
		)
	}

	/// The same pack, verified as signed by the key with id `key_id`.
	#[must_use]
	pub fn with_signer(mut self, key_id: impl Into<String>) -> Self {
		self.signer = Some(key_id.into());
		self
	}

	/// The id of the trusted key whose signature was verified, if any.
	#[must_use]
	pub fn signer(&self) -> Option<&str> {
		self.signer.as_deref()
	}

	/// Checks what pieces declare about each other and the pack.
	fn validate_links(&self) -> Result<(), PackError> {
		for piece in self.pieces.values() {
			let file = format!("{PIECES_DIR}/{}/{PIECE_MANIFEST}", piece.manifest.name);
			if let Some(missing) = piece
				.manifest
				.requires
				.iter()
				.find(|name| !self.pieces.contains_key(*name))
			{
				return Err(invalid(&file, &format!("requires unknown piece `{missing}`")));
			}
			if !piece.manifest.dependencies.is_empty() && !self.manifest.permissions.dependencies {
				return Err(invalid(
					&file,
					"declares dependencies, but `pack.toml` does not set `permissions.dependencies`",
				));
			}
			let mut path = Vec::new();
			self
				.check_cycle(&piece.manifest.name, &mut path)
				.map_err(|cycle| invalid(&file, &format!("requires itself through {cycle}")))?;
		}
		Ok(())
	}

	fn check_cycle<'a>(&'a self, name: &'a Name, path: &mut Vec<&'a Name>) -> Result<(), String> {
		if path.contains(&name) {
			path.push(name);
			let names: Vec<String> = path.iter().map(|name| format!("`{name}`")).collect();
			return Err(names.join(" → "));
		}
		path.push(name);
		for required in self
			.pieces
			.get(name)
			.map(|piece| piece.manifest.requires.as_slice())
			.unwrap_or_default()
		{
			self.check_cycle(required, path)?;
		}
		path.pop();
		Ok(())
	}

	/// `pack.toml`.
	#[must_use]
	pub fn manifest(&self) -> &PackManifest {
		&self.manifest
	}

	/// A piece by name.
	#[must_use]
	pub fn piece(&self, name: &str) -> Option<&Piece> {
		self
			.pieces
			.iter()
			.find(|(key, _)| key.as_str() == name)
			.map(|(_, piece)| piece)
	}

	/// Every piece, by name.
	pub fn pieces(&self) -> impl Iterator<Item = &Piece> {
		self.pieces.values()
	}
}

impl Piece {
	fn new(
		manifest_path: &str,
		folder: &str,
		manifest: PieceManifest,
		templates: BTreeMap<String, Vec<u8>>,
	) -> Result<Self, PackError> {
		check_schema(manifest_path, manifest.schema_version)?;
		if manifest.name.as_str() != folder {
			return Err(invalid(
				manifest_path,
				&format!("name `{}` must match its folder `{folder}`", manifest.name),
			));
		}
		let piece = Self { manifest, templates };
		piece.validate(manifest_path)?;
		Ok(piece)
	}

	/// `piece.toml`.
	#[must_use]
	pub fn manifest(&self) -> &PieceManifest {
		&self.manifest
	}

	/// Template folder for `variant`, or the default folder when the piece has no variants.
	#[must_use]
	pub(crate) fn files_dir(variant: Option<&Variant>) -> String {
		variant.map_or_else(
			|| DEFAULT_FILES_DIR.to_owned(),
			|variant| {
				variant
					.files
					.clone()
					.unwrap_or_else(|| format!("{DEFAULT_FILES_DIR}/{}", variant.name))
			},
		)
	}

	/// Templates in `dir`, keyed by path relative to `dir`.
	pub(crate) fn templates_in<'a>(&'a self, dir: &'a str) -> impl Iterator<Item = (&'a str, &'a [u8])> + 'a {
		self.templates.iter().filter_map(move |(path, bytes)| {
			path
				.strip_prefix(dir)
				.and_then(|rest| rest.strip_prefix('/'))
				.map(|rest| (rest, bytes.as_slice()))
		})
	}

	/// Examples of the piece's inputs, paired with `values`.
	#[must_use]
	pub(crate) fn examples<'a>(&'a self, values: &'a BTreeMap<String, Value>) -> Vec<Example<'a>> {
		self
			.manifest
			.inputs
			.iter()
			.filter_map(|(name, input)| {
				let example: Cow<'a, str> = match input {
					Input::String {
						example: Some(example), ..
					} => Cow::Borrowed(example),
					Input::Number {
						example: Some(example), ..
					} => Cow::Owned(example.to_string()),
					_ => return None,
				};
				Some(Example {
					input: name,
					text: example,
					value: values.get(name)?,
				})
			})
			.collect()
	}

	fn validate(&self, file: &str) -> Result<(), PackError> {
		let manifest = &self.manifest;
		for (name, input) in &manifest.inputs {
			validate_input(file, name, input)?;
		}

		let self_values: BTreeMap<String, Value> = manifest
			.inputs
			.iter()
			.filter_map(|(name, input)| match input {
				Input::String {
					example: Some(example), ..
				} => Some((name.clone(), Value::String(example.clone()))),
				Input::Number {
					example: Some(example), ..
				} => Some((name.clone(), Value::Number(*example))),
				_ => None,
			})
			.collect();
		Replacer::new(&self.examples(&self_values)).map_err(|error| invalid(file, &error.to_string()))?;

		let targets = std::iter::once(&manifest.target).chain(manifest.variants.iter().filter_map(|v| v.target.as_ref()));
		for target in targets.filter(|target| !target.is_empty()) {
			RelPath::new(target).map_err(|error| invalid(file, &format!("target: {error}")))?;
		}

		let known: BTreeSet<&str> = manifest.inputs.keys().map(String::as_str).collect();
		let check_names = |where_: &str, expr: &Expr| -> Result<(), PackError> {
			match expr
				.names()
				.into_iter()
				.find(|name| !known.contains(name) && !name.starts_with(STACK_PREFIX))
			{
				Some(name) => Err(invalid(where_, &format!("condition uses unknown input `{name}`"))),
				None => Ok(()),
			}
		};

		let mut variant_names = BTreeSet::new();
		let dirs: Vec<String> = if manifest.variants.is_empty() {
			vec![Self::files_dir(None)]
		} else {
			manifest
				.variants
				.iter()
				.map(|variant| Self::files_dir(Some(variant)))
				.collect()
		};
		for variant in &manifest.variants {
			if !variant_names.insert(&variant.name) {
				return Err(invalid(file, &format!("variant `{}` is declared twice", variant.name)));
			}
		}
		for dir in &dirs {
			if self.templates_in(dir).next().is_none() && manifest.patches.is_empty() {
				return Err(invalid(file, &format!("template folder `{dir}` is empty or missing")));
			}
		}

		for rule in &manifest.files {
			let condition = Expr::parse(&rule.when).map_err(|error| invalid(file, &error.to_string()))?;
			check_names(file, &condition)?;
			let exists = dirs
				.iter()
				.any(|dir| self.templates_in(dir).any(|(path, _)| path == rule.path));
			if !exists {
				return Err(invalid(file, &format!("file rule `{}` matches no template", rule.path)));
			}
		}

		for patch in &manifest.patches {
			RelPath::new(patch.file()).map_err(|error| invalid(file, &format!("patch file: {error}")))?;
			if let Some(when) = patch.when() {
				let condition = Expr::parse(when).map_err(|error| invalid(file, &error.to_string()))?;
				check_names(file, &condition)?;
			}
		}

		for dependency in &manifest.dependencies {
			validate_dependency(file, dependency)?;
			if let Some(unknown) = dependency.variants.iter().find(|name| !variant_names.contains(name)) {
				return Err(invalid(
					file,
					&format!("dependency `{}` names unknown variant `{unknown}`", dependency.name),
				));
			}
			if let Some(when) = &dependency.when {
				let condition = Expr::parse(when).map_err(|error| invalid(file, &error.to_string()))?;
				check_names(file, &condition)?;
			}
		}

		for (path, bytes) in &self.templates {
			if !dirs.iter().any(|dir| path.starts_with(&format!("{dir}/"))) {
				continue;
			}
			if template::is_binary(bytes) {
				continue;
			}
			let text = std::str::from_utf8(bytes).map_err(|_| invalid(path, "not valid UTF-8"))?;
			for condition in template::conditions(text).map_err(|error| invalid(path, &error.to_string()))? {
				check_names(path, &condition)?;
			}
		}
		Ok(())
	}
}

/// Names and versions end up as package-manager arguments, so only plain characters are allowed.
/// Checks every migration and returns the ast-grep rules they use, by path.
fn migration_rules(
	manifest: &PackManifest,
	files: &BTreeMap<String, Vec<u8>>,
) -> Result<BTreeMap<String, String>, PackError> {
	let mut rules = BTreeMap::new();
	for migration in &manifest.migrations {
		let fail = |reason: &str| invalid(PACK_MANIFEST, &format!("migration to {}: {reason}", migration.to));
		if migration.to > manifest.version {
			return Err(fail("`to` is newer than the pack"));
		}
		if migration.files.is_empty() {
			return Err(fail("`files` is empty"));
		}
		match (&migration.rename, &migration.ast_grep) {
			(Some(rename), None) => {
				if !crate::migrate::is_identifier(&rename.from) || !crate::migrate::is_identifier(&rename.to) {
					return Err(fail("`rename` takes plain identifiers"));
				}
			}
			(None, Some(path)) => {
				let bytes = path
					.strip_prefix(MIGRATIONS_DIR)
					.and_then(|_| files.get(path))
					.ok_or_else(|| {
						fail(&format!(
							"`ast_grep` must name a file under `{MIGRATIONS_DIR}` in the pack"
						))
					})?;
				if bytes.len() > MAX_RULE_SIZE {
					return Err(fail("the rule file is too large"));
				}
				let rule = String::from_utf8(bytes.clone()).map_err(|_| fail("the rule file is not UTF-8"))?;
				rules.insert(path.clone(), rule);
			}
			_ => return Err(fail("needs exactly one of `rename` or `ast_grep`")),
		}
	}
	Ok(rules)
}

fn validate_dependency(file: &str, dependency: &Dependency) -> Result<(), PackError> {
	let plain = |text: &str, max: usize, extra: &str| {
		(1..=max).contains(&text.len())
			&& !text.starts_with(['-', '.'])
			&& text.chars().all(|c| c.is_ascii_alphanumeric() || extra.contains(c))
	};
	if !plain(&dependency.name, 214, "@/._-") {
		return Err(invalid(
			file,
			&format!("dependency name `{}` is not a plain package name", dependency.name),
		));
	}
	if let Some(feature) = dependency.features.iter().find(|feature| !plain(feature, 64, "-_")) {
		return Err(invalid(
			file,
			&format!("dependency `{}` has an invalid feature `{feature}`", dependency.name),
		));
	}
	if !plain(&dependency.version, 64, ".^~<>=*,| +-_") {
		return Err(invalid(
			file,
			&format!(
				"dependency `{}` has an invalid version `{}`",
				dependency.name, dependency.version
			),
		));
	}
	Ok(())
}

fn validate_input(file: &str, name: &str, input: &Input) -> Result<(), PackError> {
	let valid_name = name.starts_with(|c: char| c.is_ascii_alphabetic())
		&& name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
		&& name.len() <= 64;
	if !valid_name {
		return Err(invalid(
			file,
			&format!("input name `{name}` must be letters, digits or `_`, starting with a letter"),
		));
	}
	match input {
		Input::String {
			example: Some(example),
			default,
			..
		} => {
			if example.trim().is_empty() {
				return Err(invalid(file, &format!("input `{name}` has an empty example")));
			}
			let multi_word = words(example).len() >= 2;
			if multi_word && words(example).concat().len() != example.chars().filter(|c| c.is_alphanumeric()).count() {
				return Err(invalid(file, &format!("input `{name}` example must be an identifier")));
			}
			if let Some(default) = default {
				inputs::resolve(
					&BTreeMap::from([(name.to_owned(), input.clone())]),
					&BTreeMap::from([(name.to_owned(), Value::String(default.clone()))]),
				)
				.map_err(|error| invalid(file, &format!("default: {error}")))?;
			}
		}
		Input::Enum { choices, default, .. } => {
			if choices.is_empty() {
				return Err(invalid(file, &format!("input `{name}` needs at least one choice")));
			}
			if default.as_ref().is_some_and(|default| !choices.contains(default)) {
				return Err(invalid(
					file,
					&format!("input `{name}` default is not one of its choices"),
				));
			}
		}
		Input::Number { min, max, default, .. } => {
			let (low, high) = (min.unwrap_or(i64::MIN), max.unwrap_or(i64::MAX));
			if low > high || default.is_some_and(|default| !(low..=high).contains(&default)) {
				return Err(invalid(
					file,
					&format!("input `{name}` has inconsistent bounds or default"),
				));
			}
		}
		Input::String { example: None, .. } | Input::Bool { .. } => {}
	}
	Ok(())
}

fn read_dir(
	root: &Path,
	prefix: &str,
	files: &mut BTreeMap<String, Vec<u8>>,
	total: &mut u64,
) -> Result<(), PackError> {
	let dir = if prefix.is_empty() {
		root.to_path_buf()
	} else {
		root.join(prefix)
	};
	let io = |path: &Path| {
		let path = path.to_path_buf();
		move |source| PackError::Io { path, source }
	};
	let mut entries: Vec<_> = fs::read_dir(&dir)
		.map_err(io(&dir))?
		.collect::<Result<_, _>>()
		.map_err(io(&dir))?;
	entries.sort_by_key(fs::DirEntry::file_name);

	for entry in entries {
		let path = entry.path();
		let name = entry
			.file_name()
			.into_string()
			.map_err(|_| PackError::BadPath(path.display().to_string()))?;
		if name == ".git" || name == ".DS_Store" {
			continue;
		}
		let relative = if prefix.is_empty() {
			name
		} else {
			format!("{prefix}/{name}")
		};
		let kind = entry.file_type().map_err(io(&path))?;
		if kind.is_symlink() {
			return Err(PackError::Symlink(path));
		}
		if kind.is_dir() {
			read_dir(root, &relative, files, total)?;
			continue;
		}
		let size = entry.metadata().map_err(io(&path))?.len();
		if size > MAX_FILE_SIZE {
			return Err(PackError::Limit(format!(
				"`{relative}` is larger than {MAX_FILE_SIZE} bytes"
			)));
		}
		*total += size;
		if *total > MAX_PACK_SIZE {
			return Err(PackError::Limit(format!("pack is larger than {MAX_PACK_SIZE} bytes")));
		}
		if files.len() >= MAX_FILES {
			return Err(PackError::Limit(format!("more than {MAX_FILES} files")));
		}
		files.insert(relative, fs::read(&path).map_err(io(&path))?);
	}
	Ok(())
}

fn parse<T: serde::de::DeserializeOwned>(file: &str, bytes: &[u8]) -> Result<T, PackError> {
	let text = std::str::from_utf8(bytes).map_err(|_| invalid(file, "not valid UTF-8"))?;
	toml::from_str(text).map_err(|error| PackError::Parse {
		file: file.to_owned(),
		message: error.message().to_owned(),
	})
}

fn check_schema(file: &str, version: u32) -> Result<(), PackError> {
	if version == SCHEMA_VERSION {
		Ok(())
	} else {
		Err(PackError::UnsupportedSchema {
			file: file.to_owned(),
			version,
		})
	}
}

fn invalid(file: &str, message: &str) -> PackError {
	PackError::Invalid {
		file: file.to_owned(),
		message: message.to_owned(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const PACK: &str = "schema_version = 1\nname = \"p\"\nversion = \"1.0.0\"\n";
	const PIECE: &str =
		"schema_version = 1\nname = \"thing\"\ntarget = \"src\"\n[inputs.name]\ntype = \"string\"\nexample = \"MyThing\"\n";

	fn files(extra: &[(&str, &str)]) -> BTreeMap<String, Vec<u8>> {
		let mut files = BTreeMap::from([
			("pack.toml".to_owned(), PACK.as_bytes().to_vec()),
			("pieces/thing/piece.toml".to_owned(), PIECE.as_bytes().to_vec()),
			(
				"pieces/thing/files/MyThing.ts".to_owned(),
				b"export const myThing = 1;\n".to_vec(),
			),
		]);
		for (path, text) in extra {
			files.insert((*path).to_owned(), text.as_bytes().to_vec());
		}
		files
	}

	fn error(files: BTreeMap<String, Vec<u8>>) -> String {
		Pack::from_files(files).map(|_| ()).unwrap_err().to_string()
	}

	#[test]
	fn from_files_should_load_a_valid_pack() {
		assert!(Pack::from_files(files(&[])).unwrap().piece("thing").is_some());
	}

	#[test]
	fn from_files_should_require_pack_manifest() {
		let mut files = files(&[]);
		files.remove("pack.toml");

		assert!(matches!(Pack::from_files(files), Err(PackError::MissingManifest)));
	}

	#[test]
	fn from_files_should_reject_future_schema() {
		let files = files(&[("pack.toml", "schema_version = 2\nname = \"p\"\nversion = \"1.0.0\"\n")]);

		assert!(matches!(
			Pack::from_files(files),
			Err(PackError::UnsupportedSchema { version: 2, .. })
		));
	}

	#[test]
	fn from_files_should_reject_piece_name_not_matching_folder() {
		let files = files(&[("pieces/other/piece.toml", PIECE), ("pieces/other/files/x.ts", "x")]);

		assert!(error(files).contains("must match its folder"));
	}

	#[test]
	fn from_files_should_reject_condition_on_unknown_input() {
		let files = files(&[("pieces/thing/files/MyThing.ts", "x(); // @tfverse if typo\n")]);

		assert!(error(files).contains("unknown input `typo`"));
	}

	fn piece(name: &str, extra: &str) -> (String, String) {
		(
			format!("pieces/{name}/piece.toml"),
			format!("schema_version = 1\nname = \"{name}\"\ntarget = \"src\"\n{extra}"),
		)
	}

	fn pack_with(pieces: &[(String, String)]) -> BTreeMap<String, Vec<u8>> {
		let mut files = files(&[]);
		for (path, text) in pieces {
			files.insert(path.clone(), text.as_bytes().to_vec());
			let template = path.replace("piece.toml", "files/x.ts");
			files.insert(template, b"x\n".to_vec());
		}
		files
	}

	#[test]
	fn from_files_should_reject_unknown_required_piece() {
		let files = pack_with(&[piece("a", "requires = [\"missing\"]\n")]);

		assert!(error(files).contains("requires unknown piece `missing`"));
	}

	#[test]
	fn from_files_should_reject_required_cycles() {
		let files = pack_with(&[piece("a", "requires = [\"b\"]\n"), piece("b", "requires = [\"a\"]\n")]);

		assert!(error(files).contains("requires itself"));
	}

	#[test]
	fn from_files_should_reject_dependency_names_that_look_like_flags() {
		let mut files = pack_with(&[piece(
			"a",
			"[[dependencies]]\necosystem = \"npm\"\nname = \"--registry=evil\"\nversion = \"1\"\n",
		)]);
		files.insert(
			"pack.toml".to_owned(),
			b"schema_version = 1\nname = \"p\"\nversion = \"1.0.0\"\n[permissions]\ndependencies = true\n".to_vec(),
		);

		assert!(error(files).contains("not a plain package name"));
	}

	#[test]
	fn digest_should_change_with_any_file() {
		let before = Pack::from_files(files(&[])).unwrap().digest();
		let after = Pack::from_files(files(&[("pieces/thing/files/extra.ts", "y\n")]))
			.unwrap()
			.digest();

		assert_ne!(before, after);
	}

	#[test]
	fn from_files_should_allow_stack_conditions() {
		let files = files(&[("pieces/thing/files/MyThing.ts", "x(); // @tfverse if stack.tailwind\n")]);

		assert!(Pack::from_files(files).is_ok());
	}

	#[test]
	fn from_files_should_reject_escaping_file_path() {
		let files = files(&[("pieces/thing/files/../../evil", "x")]);

		assert!(matches!(Pack::from_files(files), Err(PackError::BadPath(_))));
	}

	#[test]
	fn from_files_should_reject_unclosed_block() {
		let files = files(&[("pieces/thing/files/MyThing.ts", "// @tfverse if name\nx\n")]);

		assert!(error(files).contains("never closed"));
	}

	#[test]
	fn from_files_should_reject_file_rule_without_template() {
		let piece = format!("{PIECE}[[files]]\npath = \"missing.ts\"\nwhen = \"name\"\n");
		let files = files(&[("pieces/thing/piece.toml", &piece)]);

		assert!(error(files).contains("matches no template"));
	}

	#[test]
	fn from_files_should_reject_enum_default_outside_choices() {
		let piece = format!("{PIECE}[inputs.style]\ntype = \"enum\"\nchoices = [\"css\"]\ndefault = \"less\"\n");
		let files = files(&[("pieces/thing/piece.toml", &piece)]);

		assert!(error(files).contains("default is not one of its choices"));
	}

	#[cfg(unix)]
	#[test]
	fn load_should_reject_symlinks() {
		let dir = tempfile::tempdir().unwrap();
		std::fs::write(dir.path().join("pack.toml"), PACK).unwrap();
		std::os::unix::fs::symlink("/etc/hosts", dir.path().join("link")).unwrap();

		assert!(matches!(Pack::load(dir.path()), Err(PackError::Symlink(_))));
	}
}
