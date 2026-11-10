//! Project-relative paths that cannot escape the project root.

use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const MAX_LEN: usize = 1024;
const WINDOWS_RESERVED: [&str; 22] = [
	"con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "lpt1", "lpt2",
	"lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// A normalized, relative, `/`-separated path that stays inside its root on every OS.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelPath(String);

/// Why a path was rejected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PathError {
	/// Empty path or empty segment.
	#[error("path `{0}` is empty or has an empty segment")]
	Empty(String),
	/// Absolute path, drive prefix or `..`/`.` segment.
	#[error("path `{0}` must be relative and must not contain `.` or `..` segments")]
	Escapes(String),
	/// Backslashes, control characters or characters some OS rejects.
	#[error("path `{0}` contains a character that is not portable")]
	Unportable(String),
	/// A segment some OS reserves, like `con` on Windows.
	#[error("path `{0}` uses a name reserved on Windows")]
	Reserved(String),
	/// Longer than the supported limit.
	#[error("path `{0}` is longer than {MAX_LEN} bytes")]
	TooLong(String),
}

impl RelPath {
	/// Validates `path`.
	pub fn new(path: &str) -> Result<Self, PathError> {
		if path.len() > MAX_LEN {
			return Err(PathError::TooLong(path.to_owned()));
		}
		if path.is_empty() {
			return Err(PathError::Empty(path.to_owned()));
		}
		if path.starts_with('/') {
			return Err(PathError::Escapes(path.to_owned()));
		}
		for segment in path.split('/') {
			check_segment(path, segment)?;
		}
		Ok(Self(path.to_owned()))
	}

	/// A path from a constant known to be valid; covered by a test.
	pub(crate) fn constant(path: &'static str) -> Self {
		Self(path.to_owned())
	}

	/// A path the crate built from valid parts, such as a state file name.
	pub(crate) fn trusted(path: String) -> Self {
		debug_assert!(Self::new(&path).is_ok(), "invalid trusted path `{path}`");
		Self(path)
	}

	/// The path as written, with `/` separators.
	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// Appends `other` below this path.
	#[must_use]
	pub fn join(&self, other: &Self) -> Self {
		Self(format!("{}/{}", self.0, other.0))
	}

	/// The parent directory, if any.
	#[must_use]
	pub fn parent(&self) -> Option<Self> {
		self.0.rsplit_once('/').map(|(parent, _)| Self(parent.to_owned()))
	}

	/// The last segment.
	#[must_use]
	pub fn file_name(&self) -> &str {
		self.0.rsplit_once('/').map_or(self.0.as_str(), |(_, name)| name)
	}

	/// Every ancestor from the top down, excluding the path itself.
	pub fn ancestors(&self) -> impl Iterator<Item = Self> + '_ {
		self.0.match_indices('/').map(|(end, _)| Self(self.0[..end].to_owned()))
	}

	/// The native path below `root`.
	#[must_use]
	pub fn under(&self, root: &Path) -> PathBuf {
		self
			.0
			.split('/')
			.fold(root.to_path_buf(), |path, segment| path.join(segment))
	}
}

fn check_segment(path: &str, segment: &str) -> Result<(), PathError> {
	if segment.is_empty() {
		return Err(PathError::Empty(path.to_owned()));
	}
	if segment == "." || segment == ".." {
		return Err(PathError::Escapes(path.to_owned()));
	}
	if segment
		.chars()
		.any(|c| c.is_control() || matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
	{
		return Err(PathError::Unportable(path.to_owned()));
	}
	if segment.ends_with(['.', ' ']) {
		return Err(PathError::Unportable(path.to_owned()));
	}
	let stem = segment.split('.').next().unwrap_or(segment).to_ascii_lowercase();
	if WINDOWS_RESERVED.contains(&stem.as_str()) {
		return Err(PathError::Reserved(path.to_owned()));
	}
	Ok(())
}

impl std::str::FromStr for RelPath {
	type Err = PathError;

	fn from_str(path: &str) -> Result<Self, Self::Err> {
		Self::new(path)
	}
}

impl fmt::Display for RelPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl fmt::Debug for RelPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.0, f)
	}
}

impl Serialize for RelPath {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.0)
	}
}

impl<'de> Deserialize<'de> for RelPath {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let text = Cow::<str>::deserialize(deserializer)?;
		Self::new(&text).map_err(serde::de::Error::custom)
	}
}

impl JsonSchema for RelPath {
	fn schema_name() -> Cow<'static, str> {
		"RelPath".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({
			"type": "string",
			"description": "Relative path with `/` separators; no `.`, `..` or absolute paths.",
			"minLength": 1,
			"maxLength": MAX_LEN
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_should_accept_nested_relative_path() {
		assert_eq!(
			RelPath::new("src/ui/Button.tsx").map(|p| p.to_string()),
			Ok("src/ui/Button.tsx".to_owned())
		);
	}

	#[test]
	fn new_should_reject_parent_segment() {
		assert!(matches!(
			RelPath::new("src/../../etc/passwd"),
			Err(PathError::Escapes(_))
		));
	}

	#[test]
	fn new_should_reject_absolute_path() {
		assert!(matches!(RelPath::new("/etc/passwd"), Err(PathError::Escapes(_))));
	}

	#[test]
	fn new_should_reject_backslash() {
		assert!(matches!(RelPath::new("src\\..\\x"), Err(PathError::Unportable(_))));
	}

	#[test]
	fn new_should_reject_drive_prefix() {
		assert!(matches!(RelPath::new("C:/x"), Err(PathError::Unportable(_))));
	}

	#[test]
	fn new_should_reject_empty_segment() {
		assert!(matches!(RelPath::new("src//x"), Err(PathError::Empty(_))));
	}

	#[test]
	fn new_should_reject_windows_reserved_name() {
		assert!(matches!(RelPath::new("src/con.ts"), Err(PathError::Reserved(_))));
	}

	#[test]
	fn new_should_reject_trailing_dot() {
		assert!(matches!(RelPath::new("src/file."), Err(PathError::Unportable(_))));
	}

	#[test]
	fn constant_paths_should_be_valid() {
		use crate::brand;

		let constants = [
			brand::LOCK_FILE,
			brand::CONFIG_FILE_JSON,
			brand::CONFIG_FILE_TOML,
			"package.json",
			".env.example",
		];

		assert!(constants.iter().all(|path| RelPath::new(path).is_ok()));
	}

	#[test]
	fn ancestors_should_list_parents_top_down() {
		let path = RelPath::new("a/b/c.txt").unwrap();

		assert_eq!(
			path.ancestors().map(|a| a.to_string()).collect::<Vec<_>>(),
			["a", "a/b"]
		);
	}

	#[test]
	fn under_should_build_native_path() {
		let path = RelPath::new("a/b.txt").unwrap();

		assert_eq!(path.under(Path::new("root")), Path::new("root").join("a").join("b.txt"));
	}
}
