//! Line diffs with the histogram algorithm.

use imara_diff::{Algorithm, BasicLineDiffPrinter, Diff, InternedInput, UnifiedDiffConfig};

/// A unified diff of `before` → `after`, without file headers.
pub(crate) fn unified(before: &str, after: &str) -> String {
	let input = InternedInput::new(before, after);
	let mut diff = Diff::compute(Algorithm::Histogram, &input);
	diff.postprocess_lines(&input);
	diff
		.unified_diff(
			&BasicLineDiffPrinter(&input.interner),
			UnifiedDiffConfig::default(),
			&input,
		)
		.to_string()
}

/// The 0-based line in `base` that `line` of `current` comes from, or `None` when it was added or changed.
pub(crate) fn origin_line(base: &str, current: &str, line: u32) -> Option<u32> {
	let input = InternedInput::new(base, current);
	let mut diff = Diff::compute(Algorithm::Histogram, &input);
	diff.postprocess_lines(&input);

	let mut shift: i64 = 0;
	for hunk in diff.hunks() {
		if hunk.after.contains(&line) {
			return None;
		}
		if hunk.after.start > line {
			break;
		}
		shift += i64::from(hunk.after.end - hunk.after.start) - i64::from(hunk.before.end - hunk.before.start);
	}
	u32::try_from(i64::from(line) - shift).ok()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn unified_should_show_changed_lines() {
		assert_eq!(unified("a\nb\n", "a\nc\n"), "@@ -1,2 +1,2 @@\n a\n-b\n+c\n");
	}

	#[test]
	fn origin_line_should_map_unchanged_lines_past_insertions() {
		assert_eq!(origin_line("a\nb\nc\n", "a\nnew\nb\nc\n", 3), Some(2));
	}

	#[test]
	fn origin_line_should_report_added_lines() {
		assert_eq!(origin_line("a\nb\n", "a\nnew\nb\n", 1), None);
	}

	#[test]
	fn origin_line_should_map_lines_past_deletions() {
		assert_eq!(origin_line("a\nx\ny\nb\n", "a\nb\n", 1), Some(3));
	}
}
