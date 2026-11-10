//! Path globs: `*` within a segment, `?` for one character, `**` for any number of segments.

use std::borrow::Cow;
use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::RelPath;

const MAX_LEN: usize = 256;

/// A validated path glob.
#[derive(Clone, PartialEq, Eq)]
pub struct Glob(String);

/// Why a glob was rejected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
	"glob `{0}` must be relative, at most {MAX_LEN} characters, use `/` separators and contain no empty, `.` or `..` segments"
)]
pub struct GlobError(String);

impl Glob {
	/// Validates `pattern`.
	pub fn new(pattern: &str) -> Result<Self, GlobError> {
		let invalid = || GlobError(pattern.to_owned());
		if pattern.is_empty() || pattern.len() > MAX_LEN || pattern.starts_with('/') || pattern.contains('\\') {
			return Err(invalid());
		}
		if pattern
			.split('/')
			.any(|segment| segment.is_empty() || segment == "." || segment == "..")
		{
			return Err(invalid());
		}
		Ok(Self(pattern.to_owned()))
	}

	/// Whether `path` matches.
	#[must_use]
	pub fn matches(&self, path: &RelPath) -> bool {
		let pattern: Vec<&str> = self.0.split('/').collect();
		let segments: Vec<&str> = path.as_str().split('/').collect();
		match_segments(&pattern, &segments)
	}

	/// The pattern as written.
	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

fn match_segments(pattern: &[&str], segments: &[&str]) -> bool {
	wildcard(
		pattern,
		segments,
		|part| *part == "**",
		|part, segment| {
			let part: Vec<char> = part.chars().collect();
			let segment: Vec<char> = segment.chars().collect();
			wildcard(&part, &segment, |c| *c == '*', |p, c| *p == '?' || p == c)
		},
	)
}

/// Linear-time wildcard matching with one backtrack point, so hostile patterns cannot stall it.
fn wildcard<P, T>(pattern: &[P], text: &[T], is_star: impl Fn(&P) -> bool, matches: impl Fn(&P, &T) -> bool) -> bool {
	let (mut p, mut t) = (0, 0);
	let mut star: Option<(usize, usize)> = None;
	while t < text.len() {
		if p < pattern.len() && !is_star(&pattern[p]) && matches(&pattern[p], &text[t]) {
			p += 1;
			t += 1;
		} else if p < pattern.len() && is_star(&pattern[p]) {
			star = Some((p, t));
			p += 1;
		} else if let Some((star_p, star_t)) = star {
			p = star_p + 1;
			t = star_t + 1;
			star = Some((star_p, star_t + 1));
		} else {
			return false;
		}
	}
	pattern[p..].iter().all(is_star)
}

impl fmt::Display for Glob {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl fmt::Debug for Glob {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.0, f)
	}
}

impl Serialize for Glob {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.0)
	}
}

impl<'de> Deserialize<'de> for Glob {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let text = Cow::<str>::deserialize(deserializer)?;
		Self::new(&text).map_err(serde::de::Error::custom)
	}
}

impl JsonSchema for Glob {
	fn schema_name() -> Cow<'static, str> {
		"Glob".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({
			"type": "string",
			"description": "Relative path glob: `*` within a segment, `?` one character, `**` any segments.",
			"minLength": 1
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn matches(pattern: &str, path: &str) -> bool {
		Glob::new(pattern).unwrap().matches(&RelPath::new(path).unwrap())
	}

	#[test]
	fn double_star_should_match_any_depth() {
		assert!(matches("src/**", "src/ui/deep/Button.tsx"));
	}

	#[test]
	fn double_star_should_match_zero_segments() {
		assert!(matches("**/package.json", "package.json"));
	}

	#[test]
	fn star_should_stay_within_one_segment() {
		assert!(!matches("src/*.ts", "src/ui/index.ts"));
	}

	#[test]
	fn star_should_match_part_of_a_segment() {
		assert!(matches("**/*.config.ts", "apps/web/vite.config.ts"));
	}

	#[test]
	fn question_mark_should_match_one_character() {
		assert!(matches(".env?", ".env1") && !matches(".env?", ".env"));
	}

	#[test]
	fn star_should_match_an_empty_suffix() {
		assert!(matches("src/*", "src/a") && matches("src/a*", "src/a"));
	}

	#[test]
	fn hostile_pattern_should_finish_quickly() {
		let pattern = format!("{}b", "*a".repeat(60));
		let path = "a".repeat(200);
		let started = std::time::Instant::now();

		assert!(!matches(&pattern, &path) && started.elapsed() < std::time::Duration::from_secs(1));
	}

	#[test]
	fn new_should_reject_parent_segment() {
		assert!(Glob::new("../**").is_err());
	}
}
