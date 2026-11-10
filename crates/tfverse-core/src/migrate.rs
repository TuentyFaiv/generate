//! Pack migrations: identifier renames built in, ast-grep rules through the `ast-grep` program.

use std::fmt;
use std::time::Duration;

use serde::Deserialize;

use crate::process;

const TIMEOUT: Duration = Duration::from_secs(60);

/// Rewrites text with an ast-grep rule; a trait so tests and embedders can supply their own.
pub trait AstGrep: Send + Sync + fmt::Debug {
	/// `text` rewritten by the YAML `rule`, or `None` when the rule changes nothing.
	fn rewrite(&self, rule: &str, text: &str) -> Result<Option<String>, MigrateError>;
}

/// Why a migration could not run.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MigrateError {
	/// The `ast-grep` program is not installed.
	#[error(
		"this migration needs `ast-grep`; install it, set `ast_grep` in your user config, or update without migrations"
	)]
	Missing,
	/// ast-grep failed or answered something unexpected.
	#[error("ast-grep: {0}")]
	Failed(String),
}

/// Runs the `ast-grep` program, which reads the file on stdin and never writes to the project.
#[derive(Clone, Debug)]
pub struct AstGrepCommand {
	program: String,
	args: Vec<String>,
}

impl Default for AstGrepCommand {
	fn default() -> Self {
		Self {
			program: "ast-grep".to_owned(),
			args: Vec::new(),
		}
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Match {
	#[serde(default)]
	replacement: Option<String>,
	#[serde(default)]
	replacement_offsets: Option<Offsets>,
}

#[derive(Deserialize)]
struct Offsets {
	start: usize,
	end: usize,
}

impl AstGrepCommand {
	/// Runs `command[0]` with the remaining arguments in front of ast-grep's own; empty means `ast-grep`.
	#[must_use]
	pub fn new(command: &[String]) -> Self {
		match command.split_first() {
			Some((program, args)) => Self {
				program: program.clone(),
				args: args.to_vec(),
			},
			None => Self::default(),
		}
	}
}

impl AstGrep for AstGrepCommand {
	fn rewrite(&self, rule: &str, text: &str) -> Result<Option<String>, MigrateError> {
		let mut args = self.args.clone();
		args.extend(["scan", "--inline-rules", rule, "--stdin", "--json=compact"].map(str::to_owned));
		let output = process::run(&self.program, &args, text.as_bytes().to_vec(), TIMEOUT).map_err(|error| {
			if error.not_found() {
				MigrateError::Missing
			} else {
				MigrateError::Failed(error.to_string())
			}
		})?;
		if !output.status.success() {
			let stderr = String::from_utf8_lossy(&output.stderr);
			let first = stderr.lines().find(|line| !line.trim().is_empty()).unwrap_or_default();
			return Err(MigrateError::Failed(format!(
				"exited with {}: {}",
				output.status,
				first.trim()
			)));
		}
		let matches: Vec<Match> = serde_json::from_slice(&output.stdout)
			.map_err(|error| MigrateError::Failed(format!("unexpected output: {error}")))?;
		let mut edits: Vec<(usize, usize, String)> = matches
			.into_iter()
			.filter_map(|found| {
				let offsets = found.replacement_offsets?;
				Some((offsets.start, offsets.end, found.replacement?))
			})
			.collect();
		apply_edits(text, &mut edits).map_err(MigrateError::Failed)
	}
}

/// `text` with each `(start, end, replacement)` byte range replaced; ranges must not overlap.
fn apply_edits(text: &str, edits: &mut [(usize, usize, String)]) -> Result<Option<String>, String> {
	if edits.is_empty() {
		return Ok(None);
	}
	edits.sort_by_key(|edit| edit.0);
	let mut output = String::with_capacity(text.len());
	let mut cursor = 0;
	for (start, end, replacement) in edits.iter() {
		let valid = cursor <= *start
			&& start <= end
			&& *end <= text.len()
			&& text.is_char_boundary(*start)
			&& text.is_char_boundary(*end);
		if !valid {
			return Err(format!("overlapping or out-of-range edit at bytes {start}..{end}"));
		}
		output.push_str(&text[cursor..*start]);
		output.push_str(replacement);
		cursor = *end;
	}
	output.push_str(&text[cursor..]);
	Ok((output != text).then_some(output))
}

/// `text` with every whole-word `from` renamed to `to`, or `None` when there is none.
pub(crate) fn rename(text: &str, from: &str, to: &str) -> Option<String> {
	let word = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
	let mut output = String::with_capacity(text.len());
	let mut cursor = 0;
	let mut changed = false;
	for (at, _) in text.match_indices(from) {
		let before = text[..at].chars().next_back();
		let after = text[at + from.len()..].chars().next();
		if before.is_some_and(word) || after.is_some_and(word) || at < cursor {
			continue;
		}
		output.push_str(&text[cursor..at]);
		output.push_str(to);
		cursor = at + from.len();
		changed = true;
	}
	output.push_str(&text[cursor..]);
	changed.then_some(output)
}

/// Whether `name` can be renamed: a plain identifier.
pub(crate) fn is_identifier(name: &str) -> bool {
	(1..=128).contains(&name.len())
		&& name.starts_with(|c: char| c.is_alphabetic() || c == '_' || c == '$')
		&& name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rename_should_match_whole_words_only() {
		let text = "import { cx } from './cx'\nconst a = cx(b) + cxx + my_cx + $cx\n";

		assert_eq!(
			rename(text, "cx", "cn").as_deref(),
			Some("import { cn } from './cn'\nconst a = cn(b) + cxx + my_cx + $cx\n")
		);
		assert_eq!(rename("cxx", "cx", "cn"), None);
	}

	#[test]
	fn apply_edits_should_replace_ranges_in_order() {
		let mut edits = vec![(10, 12, "Z".to_owned()), (0, 1, "A".to_owned())];

		assert_eq!(
			apply_edits("a123456789bc", &mut edits).unwrap().as_deref(),
			Some("A123456789Z")
		);
	}

	#[test]
	fn apply_edits_should_refuse_overlaps_and_split_characters() {
		assert!(apply_edits("abcdef", &mut [(0, 3, String::new()), (2, 4, String::new())]).is_err());
		assert!(apply_edits("é", &mut [(1, 2, String::new())]).is_err());
	}

	#[cfg(unix)]
	#[test]
	fn command_should_apply_the_replacements_ast_grep_reports() {
		let json = r#"[{"replacement":"cn(\"a\", b)","replacementOffsets":{"start":10,"end":20}}]"#;
		let fake = AstGrepCommand::new(&[
			"sh".to_owned(),
			"-c".to_owned(),
			format!("cat >/dev/null; printf '%s' '{json}'"),
			"sh".to_owned(),
		]);

		let rewritten = fake.rewrite("id: x", "const a = cx(\"a\", b)\n").unwrap();

		assert_eq!(rewritten.as_deref(), Some("const a = cn(\"a\", b)\n"));
	}

	#[test]
	fn command_should_report_a_missing_program() {
		let missing = AstGrepCommand::new(&["tfverse-no-such-ast-grep".to_owned()]);

		assert!(matches!(missing.rewrite("id: x", "x"), Err(MigrateError::Missing)));
	}

	#[test]
	fn is_identifier_should_reject_patterns() {
		assert!(is_identifier("Button_2$"));
		for name in ["a.b", "2a", "", "a b", "a(b)"] {
			assert!(!is_identifier(name), "{name}");
		}
	}
}
