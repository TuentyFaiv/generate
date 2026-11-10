//! Reviewing a plan hunk by hunk, like `git add -p`: keep some changes and skip the rest.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use imara_diff::{Algorithm, Diff, InternedInput};
use serde::{Deserialize, Serialize};

use super::{Action, Change, Plan, Validated, text_diff};
use crate::model::RelPath;
use crate::template;

const CONTEXT: usize = 3;

/// One part of a change that can be kept or skipped on its own.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Hunk {
	/// Position in the change, from 0.
	pub index: usize,
	/// `@@ -start,count +start,count @@`, or `binary` for a binary file.
	pub header: String,
	/// The changed lines with up to three lines of context, prefixed with ` `, `-` or `+`.
	pub lines: String,
}

/// Which parts of one file's change to keep.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "PickForm")]
pub enum Pick {
	/// The whole change.
	All,
	/// Nothing: the file stays as it is.
	None,
	/// Only these hunks.
	Hunks(BTreeSet<usize>),
}

/// What to keep of each file; files not listed keep their whole change.
pub type Selection = BTreeMap<RelPath, Pick>;

/// `"all"`, `"none"` or a list of hunk indexes.
#[derive(Deserialize)]
#[serde(untagged)]
enum PickForm {
	Word(String),
	Hunks(BTreeSet<usize>),
}

impl TryFrom<PickForm> for Pick {
	type Error = String;

	fn try_from(form: PickForm) -> Result<Self, Self::Error> {
		match form {
			PickForm::Word(word) if word == "all" => Ok(Self::All),
			PickForm::Word(word) if word == "none" => Ok(Self::None),
			PickForm::Word(word) => Err(format!("expected `all`, `none` or hunk indexes, found `{word}`")),
			PickForm::Hunks(hunks) => Ok(Self::Hunks(hunks)),
		}
	}
}

/// Why a selection does not fit the plan.
#[derive(Debug, PartialEq, Eq, thiserror::Error, Serialize)]
#[non_exhaustive]
pub enum SelectError {
	/// The plan does not change the file.
	#[error("the plan does not change `{0}`")]
	UnknownPath(RelPath),
	/// The change has fewer hunks.
	#[error("`{path}` has no hunk {index}")]
	UnknownHunk {
		/// The file.
		path: RelPath,
		/// The missing hunk.
		index: usize,
	},
	/// Binary files are kept or skipped whole.
	#[error("`{0}` is binary; keep all or none of it")]
	Binary(RelPath),
}

/// A changed region: `before` lines replaced by `after` lines, 0-based.
#[derive(Clone, Debug)]
struct Region {
	before: Range<usize>,
	after: Range<usize>,
}

impl Change {
	/// The parts of this change that can be kept or skipped one by one.
	#[must_use]
	pub fn hunks(&self) -> Vec<Hunk> {
		if self.action == Action::Unchanged {
			return Vec::new();
		}
		let (before, after) = self.texts();
		if template::is_binary(before) || template::is_binary(after) {
			return vec![Hunk {
				index: 0,
				header: "binary".to_owned(),
				lines: String::new(),
			}];
		}
		let (before, after) = (String::from_utf8_lossy(before), String::from_utf8_lossy(after));
		let (old, new) = (lines(&before), lines(&after));
		regions(&before, &after)
			.iter()
			.enumerate()
			.map(|(index, region)| render(index, region, &old, &new))
			.collect()
	}

	/// The content before and after this change; empty for a file that does not exist on that side.
	fn texts(&self) -> (&[u8], &[u8]) {
		let after = if self.action == Action::Delete {
			&[][..]
		} else {
			&self.contents
		};
		(self.before.as_deref().unwrap_or_default(), after)
	}
}

impl Plan<Validated> {
	/// Keeps only the parts of the plan that `selection` picks; skipped files are not touched.
	///
	/// A skipped new file is also left out of the lockfile, so the next update offers it again. The
	/// whole selection is checked first, so on error the plan is unchanged.
	pub fn select(&mut self, selection: &Selection) -> Result<(), SelectError> {
		let mut decisions = Vec::new();
		for (path, pick) in selection {
			let index = self
				.state
				.changes
				.iter()
				.position(|change| change.path == *path)
				.ok_or_else(|| SelectError::UnknownPath(path.clone()))?;
			if let Some(decision) = decide(&self.state.changes[index], pick)? {
				decisions.push((path.clone(), decision));
			}
		}
		for (path, decision) in decisions {
			let Some(index) = self.state.changes.iter().position(|change| change.path == path) else {
				continue;
			};
			match decision {
				Decision::Partial(contents) => {
					let change = &mut self.state.changes[index];
					let before = change.before.clone().unwrap_or_default();
					change.conflicts = markers(&contents).saturating_sub(markers(&before));
					change.diff = text_diff(&before, &contents);
					change.contents = contents;
				}
				Decision::Skip => {
					let action = self.state.changes.remove(index).action;
					for instance in &mut self.record.instances {
						match action {
							Action::Create => instance.files.retain(|file| file.path != path),
							Action::Modify => instance.patches.retain(|patch| patch.file != path),
							Action::Overwrite | Action::Delete | Action::Unchanged => {}
						}
					}
				}
			}
		}
		Ok(())
	}
}

/// What a pick does to a change that it does not keep whole.
enum Decision {
	/// Leave the file as it is.
	Skip,
	/// Write these contents instead.
	Partial(Vec<u8>),
}

/// What `pick` does to `change`, or `None` when it keeps the whole change.
fn decide(change: &Change, pick: &Pick) -> Result<Option<Decision>, SelectError> {
	let hunks = change.hunks().len();
	let keep = match pick {
		Pick::All => return Ok(None),
		Pick::None => BTreeSet::new(),
		Pick::Hunks(indexes) => indexes.clone(),
	};
	if let Some(index) = keep.iter().find(|index| **index >= hunks) {
		return Err(SelectError::UnknownHunk {
			path: change.path.clone(),
			index: *index,
		});
	}
	if keep.len() == hunks {
		return Ok(None);
	}
	if keep.is_empty() {
		return Ok(Some(Decision::Skip));
	}
	let (before, after) = change.texts();
	if template::is_binary(before) || template::is_binary(after) {
		return Err(SelectError::Binary(change.path.clone()));
	}
	Ok(Some(Decision::Partial(partial(
		&String::from_utf8_lossy(before),
		&String::from_utf8_lossy(after),
		&keep,
	))))
}

fn lines(text: &str) -> Vec<&str> {
	text.split_inclusive('\n').collect()
}

fn regions(before: &str, after: &str) -> Vec<Region> {
	let input = InternedInput::new(before, after);
	let mut diff = Diff::compute(Algorithm::Histogram, &input);
	diff.postprocess_lines(&input);
	let index = |line: u32| usize::try_from(line).unwrap_or(usize::MAX);
	diff
		.hunks()
		.map(|hunk| Region {
			before: index(hunk.before.start)..index(hunk.before.end),
			after: index(hunk.after.start)..index(hunk.after.end),
		})
		.collect()
}

fn render(index: usize, region: &Region, old: &[&str], new: &[&str]) -> Hunk {
	let leading = region.before.start.saturating_sub(CONTEXT)..region.before.start;
	let trailing = region.before.end..(region.before.end + CONTEXT).min(old.len());
	let mut lines = String::new();
	let mut push = |prefix: char, line: &str| {
		lines.push(prefix);
		lines.push_str(line);
		if !line.ends_with('\n') {
			lines.push('\n');
		}
	};
	for line in &old[leading.clone()] {
		push(' ', line);
	}
	for line in &old[region.before.clone()] {
		push('-', line);
	}
	for line in &new[region.after.clone()] {
		push('+', line);
	}
	for line in &old[trailing.clone()] {
		push(' ', line);
	}
	let context = leading.len() + trailing.len();
	let removed = region.before.len() + context;
	let added = region.after.len() + context;
	let after_start = region.after.start - leading.len();
	Hunk {
		index,
		header: format!(
			"@@ -{},{removed} +{},{added} @@",
			start(leading.start, removed),
			start(after_start, added)
		),
		lines,
	}
}

/// The 1-based start line of a unified hunk side, or 0 when that side is empty.
fn start(line: usize, count: usize) -> usize {
	if count == 0 { line } else { line + 1 }
}

/// `before` with only the hunks in `keep` applied.
fn partial(before: &str, after: &str, keep: &BTreeSet<usize>) -> Vec<u8> {
	let (old, new) = (lines(before), lines(after));
	let mut text = String::with_capacity(before.len().max(after.len()));
	let mut cursor = 0;
	for (index, region) in regions(before, after).iter().enumerate() {
		text.extend(old[cursor..region.before.start].iter().copied());
		if keep.contains(&index) {
			text.extend(new[region.after.clone()].iter().copied());
		} else {
			text.extend(old[region.before.clone()].iter().copied());
		}
		cursor = region.before.end;
	}
	text.extend(old[cursor..].iter().copied());
	text.into_bytes()
}

fn markers(text: &[u8]) -> usize {
	String::from_utf8_lossy(text)
		.lines()
		.filter(|line| line.starts_with("<<<<<<< "))
		.count()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn partial_should_apply_only_kept_hunks() {
		let before = "a\nb\nc\nd\ne\nf\ng\nh\ni\n";
		let after = "A\nb\nc\nd\ne\nf\ng\nh\nI\n";

		assert_eq!(
			partial(before, after, &BTreeSet::from([1])),
			b"a\nb\nc\nd\ne\nf\ng\nh\nI\n"
		);
		assert_eq!(
			partial(before, after, &BTreeSet::from([0])),
			b"A\nb\nc\nd\ne\nf\ng\nh\ni\n"
		);
	}

	#[test]
	fn render_should_show_context_and_a_unified_header() {
		let (before, after) = ("a\nb\nc\nd\ne\n", "a\nb\nc\nX\ne\n");
		let region = regions(before, after).remove(0);

		let hunk = render(0, &region, &lines(before), &lines(after));

		assert_eq!(hunk.header, "@@ -1,5 +1,5 @@");
		assert_eq!(hunk.lines, " a\n b\n c\n-d\n+X\n e\n");
	}

	#[test]
	fn render_should_number_an_empty_side_from_zero() {
		let region = regions("", "new\n").remove(0);

		let hunk = render(0, &region, &[], &lines("new\n"));

		assert_eq!(
			(hunk.header.as_str(), hunk.lines.as_str()),
			("@@ -0,0 +1,1 @@", "+new\n")
		);
	}

	#[test]
	fn pick_should_read_words_and_hunk_lists() {
		let picks: Vec<Pick> = serde_json::from_str(r#"["all", "none", [0, 2]]"#).unwrap();

		assert_eq!(picks, [Pick::All, Pick::None, Pick::Hunks(BTreeSet::from([0, 2]))]);
		assert!(serde_json::from_str::<Pick>(r#""some""#).is_err());
	}
}
