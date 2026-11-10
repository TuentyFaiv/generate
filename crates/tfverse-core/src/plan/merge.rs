//! Three-way line merges: the user's edits and a pack's new version, both made from the same original.

use std::ops::Range;

use imara_diff::{Algorithm, Diff, InternedInput};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// What to do where the user's edits and the new version change the same lines.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OnConflict {
	/// Reject the plan and list the conflicting files.
	#[default]
	Refuse,
	/// Write both versions between Git-style conflict markers.
	Markers,
	/// Keep the user's lines.
	Ours,
	/// Take the new version's lines.
	Theirs,
}

/// A merged text and how many regions conflicted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Merged {
	/// The merged text.
	pub text: String,
	/// Regions both sides changed differently.
	pub conflicts: usize,
}

/// Labels for the conflict markers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Labels<'a> {
	/// The user's side.
	pub ours: &'a str,
	/// The new version.
	pub theirs: &'a str,
}

/// Merges `ours` and `theirs`, both edited from `base`; changes to the same or adjacent lines conflict.
pub(crate) fn merge(base: &str, ours: &str, theirs: &str, on_conflict: OnConflict, labels: Labels<'_>) -> Merged {
	let base_lines: Vec<&str> = base.split_inclusive('\n').collect();
	let our_lines: Vec<&str> = ours.split_inclusive('\n').collect();
	let their_lines: Vec<&str> = theirs.split_inclusive('\n').collect();
	let mut ours_hunks = hunks(base, ours).into_iter().peekable();
	let mut theirs_hunks = hunks(base, theirs).into_iter().peekable();

	let mut text = String::with_capacity(ours.len().max(theirs.len()));
	let (mut cursor, mut our_shift, mut their_shift) = (0_i64, 0_i64, 0_i64);
	let mut conflicts = 0;
	loop {
		let start = match (ours_hunks.peek(), theirs_hunks.peek()) {
			(None, None) => break,
			(Some(a), Some(b)) => a.base.start.min(b.base.start),
			(Some(a), None) => a.base.start,
			(None, Some(b)) => b.base.start,
		};
		let mut end = start;
		let (mut our_delta, mut their_delta) = (0_i64, 0_i64);
		let (mut ours_changed, mut theirs_changed) = (false, false);
		loop {
			let mut grew = false;
			if let Some(hunk) = ours_hunks.next_if(|hunk| hunk.base.start <= end) {
				end = end.max(hunk.base.end);
				our_delta += hunk.delta();
				ours_changed = true;
				grew = true;
			}
			if let Some(hunk) = theirs_hunks.next_if(|hunk| hunk.base.start <= end) {
				end = end.max(hunk.base.end);
				their_delta += hunk.delta();
				theirs_changed = true;
				grew = true;
			}
			if !grew {
				break;
			}
		}

		push(&mut text, &base_lines, cursor..start);
		let ours_region = (start + our_shift)..(end + our_shift + our_delta);
		let theirs_region = (start + their_shift)..(end + their_shift + their_delta);
		let same = slice(&our_lines, ours_region.clone()) == slice(&their_lines, theirs_region.clone());
		match (ours_changed, theirs_changed) {
			(true, false) => push(&mut text, &our_lines, ours_region),
			(false, _) => push(&mut text, &their_lines, theirs_region),
			(true, true) if same => push(&mut text, &our_lines, ours_region),
			(true, true) => {
				conflicts += 1;
				match on_conflict {
					OnConflict::Ours => push(&mut text, &our_lines, ours_region),
					OnConflict::Theirs => push(&mut text, &their_lines, theirs_region),
					OnConflict::Refuse | OnConflict::Markers => {
						marker(&mut text, '<', labels.ours);
						push(&mut text, &our_lines, ours_region);
						marker(&mut text, '|', "original");
						push(&mut text, &base_lines, start..end);
						marker(&mut text, '=', "");
						push(&mut text, &their_lines, theirs_region);
						marker(&mut text, '>', labels.theirs);
					}
				}
			}
		}
		cursor = end;
		our_shift += our_delta;
		their_shift += their_delta;
	}
	push(&mut text, &base_lines, cursor..len(&base_lines));
	Merged { text, conflicts }
}

/// A changed region: `base` lines replaced by `side` lines.
#[derive(Clone, Debug)]
struct Hunk {
	base: Range<i64>,
	side: Range<i64>,
}

impl Hunk {
	fn delta(&self) -> i64 {
		(self.side.end - self.side.start) - (self.base.end - self.base.start)
	}
}

fn hunks(base: &str, side: &str) -> Vec<Hunk> {
	let input = InternedInput::new(base, side);
	let mut diff = Diff::compute(Algorithm::Histogram, &input);
	diff.postprocess_lines(&input);
	diff
		.hunks()
		.map(|hunk| Hunk {
			base: i64::from(hunk.before.start)..i64::from(hunk.before.end),
			side: i64::from(hunk.after.start)..i64::from(hunk.after.end),
		})
		.collect()
}

fn slice<'a>(lines: &'a [&'a str], range: Range<i64>) -> &'a [&'a str] {
	let start = usize::try_from(range.start).unwrap_or(0).min(lines.len());
	let end = usize::try_from(range.end).unwrap_or(0).clamp(start, lines.len());
	&lines[start..end]
}

fn push(text: &mut String, lines: &[&str], range: Range<i64>) {
	for line in slice(lines, range) {
		text.push_str(line);
	}
}

/// A conflict marker line, ending the previous line first if it had no newline.
fn marker(text: &mut String, symbol: char, label: &str) {
	if !text.is_empty() && !text.ends_with('\n') {
		text.push('\n');
	}
	text.extend(std::iter::repeat_n(symbol, 7));
	if !label.is_empty() {
		text.push(' ');
		text.push_str(label);
	}
	text.push('\n');
}

fn len(lines: &[&str]) -> i64 {
	i64::try_from(lines.len()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
	use super::*;

	const LABELS: Labels<'static> = Labels {
		ours: "yours",
		theirs: "ui 1.1.0",
	};

	fn merged(base: &str, ours: &str, theirs: &str, on_conflict: OnConflict) -> Merged {
		merge(base, ours, theirs, on_conflict, LABELS)
	}

	#[test]
	fn merge_should_keep_both_sides_when_they_touch_different_lines() {
		let result = merged(
			"a\nb\nc\nd\ne\n",
			"a\nB\nc\nd\ne\n",
			"a\nb\nc\nd\nE\n",
			OnConflict::Refuse,
		);

		assert_eq!(
			result,
			Merged {
				text: "a\nB\nc\nd\nE\n".to_owned(),
				conflicts: 0
			}
		);
	}

	#[test]
	fn merge_should_take_identical_changes_once() {
		let result = merged("a\nb\n", "a\nx\n", "a\nx\n", OnConflict::Refuse);

		assert_eq!(result.text, "a\nx\n");
		assert_eq!(result.conflicts, 0);
	}

	#[test]
	fn merge_should_keep_insertions_and_deletions_on_both_sides() {
		let result = merged(
			"one\ntwo\nthree\nfour\nfive\nsix\n",
			"zero\none\ntwo\nthree\nfour\nfive\nsix\n",
			"one\ntwo\nthree\nfour\nsix\n",
			OnConflict::Refuse,
		);

		assert_eq!(result.text, "zero\none\ntwo\nthree\nfour\nsix\n");
		assert_eq!(result.conflicts, 0);
	}

	#[test]
	fn merge_should_mark_lines_both_sides_changed() {
		let result = merged("a\nb\nc\n", "a\nmine\nc\n", "a\ntheirs\nc\n", OnConflict::Markers);

		assert_eq!(
			result.text,
			"a\n<<<<<<< yours\nmine\n||||||| original\nb\n=======\ntheirs\n>>>>>>> ui 1.1.0\nc\n"
		);
		assert_eq!(result.conflicts, 1);
	}

	#[test]
	fn merge_should_resolve_conflicts_by_side() {
		let ours = merged("a\nb\nc\n", "a\nmine\nc\n", "a\ntheirs\nc\n", OnConflict::Ours);
		let theirs = merged("a\nb\nc\n", "a\nmine\nc\n", "a\ntheirs\nc\n", OnConflict::Theirs);

		assert_eq!((ours.text.as_str(), ours.conflicts), ("a\nmine\nc\n", 1));
		assert_eq!((theirs.text.as_str(), theirs.conflicts), ("a\ntheirs\nc\n", 1));
	}

	#[test]
	fn merge_should_treat_adjacent_changes_as_a_conflict() {
		let result = merged("a\nb\nc\n", "a\nB\nc\n", "a\nb\nC\n", OnConflict::Theirs);

		assert_eq!(result.conflicts, 1);
	}

	#[test]
	fn merge_should_return_ours_when_theirs_is_unchanged() {
		let result = merged("a\nb", "a\nb\nmore", "a\nb", OnConflict::Refuse);

		assert_eq!(result.text, "a\nb\nmore");
	}

	#[test]
	fn merge_should_end_a_line_before_a_marker() {
		let result = merged("a\nb", "a\nmine", "a\ntheirs", OnConflict::Markers);

		assert_eq!(
			result.text,
			"a\n<<<<<<< yours\nmine\n||||||| original\nb\n=======\ntheirs\n>>>>>>> ui 1.1.0\n"
		);
	}
}
