//! Adds CSS custom properties to a block such as `:root`, creating the block when it is missing.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::patch::{Patch, PatchError, Patched, Patcher, line_ending};

/// The built-in CSS variables patcher; values the file already sets are kept.
#[derive(Clone, Copy, Debug, Default)]
pub struct CssVarsPatcher;

impl Patcher for CssVarsPatcher {
	fn apply(&self, text: &str, patch: &Patch) -> Result<Patched, PatchError> {
		let Patch::CssVars { selector, vars, .. } = patch else {
			return Err(PatchError::Invalid(
				"CSS variables patcher received another patch kind".to_owned(),
			));
		};
		check(selector, vars)?;
		let ending = line_ending(text);
		let Some(block) = find_block(text, selector) else {
			let declarations = declarations(vars, "  ", ending);
			let lead = if text.is_empty() || text.ends_with('\n') {
				""
			} else {
				ending
			};
			let inserted = format!("{lead}{ending}{selector} {{{ending}{declarations}}}{ending}");
			return Ok(Patched::Changed {
				text: format!("{text}{inserted}"),
				inserted,
			});
		};
		let body = &text[block.open + 1..block.close];
		let missing: BTreeMap<String, String> = vars
			.iter()
			.filter(|(name, _)| !declares(body, name))
			.map(|(name, value)| (name.clone(), value.clone()))
			.collect();
		if missing.is_empty() {
			return Ok(Patched::AlreadyApplied {
				inserted: String::new(),
			});
		}
		let indent = body.lines().find(|line| !line.trim().is_empty()).map_or_else(
			|| format!("{}  ", block.indent),
			|line| line.chars().take_while(|c| *c == ' ' || *c == '\t').collect(),
		);
		let line_start = text[..block.close].rfind('\n').map_or(0, |at| at + 1);
		let closing_alone = text[line_start..block.close].trim().is_empty();
		let (at, inserted) = if closing_alone {
			(line_start, declarations(&missing, &indent, ending))
		} else {
			(
				block.close,
				format!("{ending}{}", declarations(&missing, &indent, ending)),
			)
		};
		Ok(Patched::Changed {
			text: format!("{}{inserted}{}", &text[..at], &text[at..]),
			inserted,
		})
	}
}

/// A block's `{` and matching `}` byte positions, and its header's indentation.
struct Block {
	open: usize,
	close: usize,
	indent: String,
}

/// The first block whose header is exactly `selector`.
fn find_block(text: &str, selector: &str) -> Option<Block> {
	let mut offset = 0;
	for line in text.split_inclusive('\n') {
		let trimmed = line.trim_start();
		if let Some(rest) = trimmed.strip_prefix(selector)
			&& rest.trim_start().starts_with('{')
		{
			let open = offset + (line.len() - trimmed.len()) + selector.len() + (rest.len() - rest.trim_start().len());
			let close = matching_brace(text, open)?;
			let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
			return Some(Block { open, close, indent });
		}
		offset += line.len();
	}
	None
}

/// The `}` closing the `{` at `open`, skipping strings and comments.
fn matching_brace(text: &str, open: usize) -> Option<usize> {
	let bytes = text.as_bytes();
	let mut depth = 0_usize;
	let mut index = open;
	while index < bytes.len() {
		match bytes[index] {
			b'/' if bytes.get(index + 1) == Some(&b'*') => {
				index = text[index + 2..].find("*/").map(|end| index + 2 + end + 1)?;
			}
			quote @ (b'"' | b'\'') => {
				index += 1 + text[index + 1..].find(quote as char)?;
			}
			b'{' => depth += 1,
			b'}' => {
				depth -= 1;
				if depth == 0 {
					return Some(index);
				}
			}
			_ => {}
		}
		index += 1;
	}
	None
}

/// Whether `body` already sets `--name`.
fn declares(body: &str, name: &str) -> bool {
	let property = format!("--{name}");
	body.match_indices(&property).any(|(at, _)| {
		let before = body[..at].chars().next_back();
		let after = body[at + property.len()..].trim_start();
		before.is_none_or(|c| c.is_whitespace() || c == ';' || c == '{') && after.starts_with(':')
	})
}

fn declarations(vars: &BTreeMap<String, String>, indent: &str, ending: &str) -> String {
	vars.iter().fold(String::new(), |mut text, (name, value)| {
		let _ = write!(text, "{indent}--{name}: {value};{ending}");
		text
	})
}

/// Refuses names and values that could escape the declaration they are written into.
fn check(selector: &str, vars: &BTreeMap<String, String>) -> Result<(), PatchError> {
	let unsafe_text =
		|text: &str| text.contains(['{', '}', ';', '\n', '\r']) || text.contains("/*") || text.contains("*/");
	if selector.trim().is_empty() || unsafe_text(selector) {
		return Err(PatchError::Invalid(format!("invalid CSS selector `{selector}`")));
	}
	for (name, value) in vars {
		if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
			return Err(PatchError::Invalid(format!("invalid CSS variable name `{name}`")));
		}
		if value.trim().is_empty() || unsafe_text(value) {
			return Err(PatchError::Invalid(format!("invalid value for `--{name}`")));
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::RelPath;

	fn patch(selector: &str, vars: &[(&str, &str)]) -> Patch {
		Patch::CssVars {
			file: RelPath::new("src/app.css").unwrap(),
			selector: selector.to_owned(),
			vars: vars
				.iter()
				.map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
				.collect(),
		}
	}

	fn apply(text: &str, patch: &Patch) -> Patched {
		CssVarsPatcher.apply(text, patch).unwrap()
	}

	#[test]
	fn apply_should_add_missing_variables_and_keep_existing_ones() {
		let css = "@import \"tailwindcss\";\n\n:root {\n  --radius: 0.5rem;\n  --sidebar: red;\n}\n";

		let result = apply(
			css,
			&patch(":root", &[("sidebar", "white"), ("sidebar-border", "gray")]),
		);

		assert_eq!(
			result,
			Patched::Changed {
				text:
					"@import \"tailwindcss\";\n\n:root {\n  --radius: 0.5rem;\n  --sidebar: red;\n  --sidebar-border: gray;\n}\n"
						.to_owned(),
				inserted: "  --sidebar-border: gray;\n".to_owned(),
			}
		);
	}

	#[test]
	fn apply_should_create_a_missing_block() {
		let result = apply(":root {\n  --a: 1;\n}", &patch(".dark", &[("a", "2")]));

		assert_eq!(
			result,
			Patched::Changed {
				text: ":root {\n  --a: 1;\n}\n\n.dark {\n  --a: 2;\n}\n".to_owned(),
				inserted: "\n\n.dark {\n  --a: 2;\n}\n".to_owned(),
			}
		);
	}

	#[test]
	fn apply_should_find_nested_blocks_and_skip_braces_in_comments() {
		let css = "@layer base {\n  .dark {\n    /* { not a block } */\n    --a: 1;\n  }\n}\n";

		let Patched::Changed { text, .. } = apply(css, &patch(".dark", &[("b", "2")])) else {
			panic!("expected a change");
		};

		assert_eq!(
			text,
			"@layer base {\n  .dark {\n    /* { not a block } */\n    --a: 1;\n    --b: 2;\n  }\n}\n"
		);
	}

	#[test]
	fn apply_should_report_nothing_to_do() {
		assert_eq!(
			apply(
				"@theme inline {\n  --color-x: var(--x);\n}\n",
				&patch("@theme inline", &[("color-x", "blue")])
			),
			Patched::AlreadyApplied {
				inserted: String::new()
			}
		);
	}

	#[test]
	fn apply_should_refuse_values_that_escape_the_declaration() {
		for vars in [
			[("a", "red; } body { color: red")],
			[("a b", "red")],
			[("a", "red /* x")],
		] {
			assert!(
				CssVarsPatcher.apply(":root {}", &patch(":root", &vars)).is_err(),
				"{vars:?}"
			);
		}
	}
}
