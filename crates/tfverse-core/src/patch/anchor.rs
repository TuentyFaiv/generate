//! Inserts text next to a unique anchor line, matching its indentation; works in any language.

use crate::model::Position;
use crate::patch::Patch;
use crate::patch::{PatchError, Patched, Patcher, line_ending};

/// The built-in anchor patcher.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnchorPatcher;

impl Patcher for AnchorPatcher {
	fn apply(&self, text: &str, patch: &Patch) -> Result<Patched, PatchError> {
		let Patch::Anchor {
			anchor,
			position,
			insert,
			..
		} = patch
		else {
			return Err(PatchError::Invalid(
				"anchor patcher received another patch kind".to_owned(),
			));
		};

		let lines: Vec<&str> = text.split_inclusive('\n').collect();
		let matches: Vec<usize> = lines
			.iter()
			.enumerate()
			.filter(|(_, line)| line.contains(anchor.as_str()))
			.map(|(i, _)| i)
			.collect();
		let index = match matches.as_slice() {
			[] => return Err(PatchError::MissingAnchor(anchor.clone())),
			[index] => *index,
			_ => {
				return Err(PatchError::AmbiguousAnchor {
					anchor: anchor.clone(),
					count: matches.len(),
				});
			}
		};

		let anchor_line = lines[index];
		let indent: String = anchor_line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
		let ending = line_ending(text);
		let inserted: String = insert
			.lines()
			.map(|line| {
				if line.is_empty() {
					ending.to_owned()
				} else {
					format!("{indent}{line}{ending}")
				}
			})
			.collect();

		if text.contains(&inserted) {
			return Ok(Patched::AlreadyApplied { inserted });
		}

		let mut output = String::with_capacity(text.len() + inserted.len() + ending.len());
		for (i, line) in lines.iter().enumerate() {
			if i == index && *position == Position::Before {
				output.push_str(&inserted);
			}
			output.push_str(line);
			if i == index && *position == Position::After {
				if !line.ends_with('\n') {
					output.push_str(ending);
				}
				output.push_str(&inserted);
			}
		}
		Ok(Patched::Changed { text: output, inserted })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::RelPath;

	fn patch(anchor: &str, position: Position, insert: &str) -> Patch {
		Patch::Anchor {
			file: RelPath::constant("src/routes.ts"),
			anchor: anchor.to_owned(),
			position,
			insert: insert.to_owned(),
		}
	}

	fn apply(text: &str, patch: &Patch) -> Result<Patched, PatchError> {
		AnchorPatcher.apply(text, patch)
	}

	#[test]
	fn after_should_insert_with_anchor_indentation() {
		let text = "const routes = [\n  // @tfverse:routes\n];\n";
		let result = apply(text, &patch("@tfverse:routes", Position::After, "{ path: '/card' },"));

		assert_eq!(
			result,
			Ok(Patched::Changed {
				text: "const routes = [\n  // @tfverse:routes\n  { path: '/card' },\n];\n".to_owned(),
				inserted: "  { path: '/card' },\n".to_owned()
			})
		);
	}

	#[test]
	fn before_should_insert_above_anchor() {
		let result = apply("a\n// end\n", &patch("// end", Position::Before, "b"));

		assert!(matches!(result, Ok(Patched::Changed { text, .. }) if text == "a\nb\n// end\n"));
	}

	#[test]
	fn second_apply_should_be_idempotent() {
		let patch = patch("// here", Position::After, "x();");
		let Ok(Patched::Changed { text, .. }) = apply("// here\n", &patch) else {
			panic!("first apply must change")
		};

		assert!(matches!(apply(&text, &patch), Ok(Patched::AlreadyApplied { .. })));
	}

	#[test]
	fn missing_anchor_should_fail() {
		assert_eq!(
			apply("nothing\n", &patch("// here", Position::After, "x")),
			Err(PatchError::MissingAnchor("// here".to_owned()))
		);
	}

	#[test]
	fn duplicated_anchor_should_fail() {
		assert!(matches!(
			apply("// here\n// here\n", &patch("// here", Position::After, "x")),
			Err(PatchError::AmbiguousAnchor { count: 2, .. })
		));
	}

	#[test]
	fn crlf_files_should_keep_crlf() {
		let result = apply("// here\r\nend\r\n", &patch("// here", Position::After, "x"));

		assert!(matches!(result, Ok(Patched::Changed { text, .. }) if text == "// here\r\nx\r\nend\r\n"));
	}
}
