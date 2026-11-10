//! Comment directives (`@tfverse if`, `else`, `end`) that keep or drop lines, in any comment style.

use super::expr::{Expr, ExprError};
use crate::model::Values;

const MARKER: &str = "@tfverse";
const OPENERS: [&str; 7] = ["{/*", "<!--", "/*", "//", "--", "#", ";"];
const CLOSERS: [&str; 3] = ["*/}", "-->", "*/"];
const MAX_DEPTH: usize = 16;

/// Why a template's directives are invalid.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BlockError {
	/// `@tfverse` outside a comment.
	#[error("line {0}: `@tfverse` must be inside a comment")]
	NotInComment(usize),
	/// Unknown directive word.
	#[error("line {line}: unknown directive `@tfverse {word}` (expected `if`, `else` or `end`)")]
	Unknown {
		/// 1-based line.
		line: usize,
		/// The directive as written.
		word: String,
	},
	/// `else` or `end` after code on the same line.
	#[error("line {0}: only `@tfverse if` may follow code on the same line")]
	InlineBlock(usize),
	/// `else` or `end` without an open `if`.
	#[error("line {0}: directive has no matching `@tfverse if`")]
	Unmatched(usize),
	/// A second `else` in one block.
	#[error("line {0}: block already has an `@tfverse else`")]
	DoubleElse(usize),
	/// An `if` that is never closed.
	#[error("line {0}: `@tfverse if` is never closed with `@tfverse end`")]
	Unclosed(usize),
	/// Blocks nested too deeply.
	#[error("line {0}: blocks are nested more than {MAX_DEPTH} levels deep")]
	TooDeep(usize),
	/// Invalid condition.
	#[error("line {line}: {source}")]
	Condition {
		/// 1-based line.
		line: usize,
		/// Parse error.
		source: ExprError,
	},
}

#[derive(Debug)]
enum Line<'a> {
	Text(&'a str),
	Inline { code: String, condition: Expr },
	If(Expr),
	Else,
	End,
}

/// Every condition in `text`, validating the directive structure.
pub fn conditions(text: &str) -> Result<Vec<Expr>, BlockError> {
	let lines = parse(text)?;
	Ok(
		lines
			.into_iter()
			.filter_map(|line| match line {
				Line::If(condition) | Line::Inline { condition, .. } => Some(condition),
				Line::Text(_) | Line::Else | Line::End => None,
			})
			.collect(),
	)
}

/// Keeps or drops lines of `text` according to its directives and `values`.
pub fn process(text: &str, values: &Values) -> Result<String, BlockError> {
	struct Frame {
		parent: bool,
		matched: bool,
		active: bool,
	}

	let mut output = String::with_capacity(text.len());
	let mut stack: Vec<Frame> = Vec::new();

	for line in parse(text)? {
		let active = stack.last().is_none_or(|frame| frame.active);
		match line {
			Line::Text(text) if active => output.push_str(text),
			Line::Inline { code, condition } if active && condition.eval(values) => output.push_str(&code),
			Line::Text(_) | Line::Inline { .. } => {}
			Line::If(condition) => {
				let matched = condition.eval(values);
				stack.push(Frame {
					parent: active,
					matched,
					active: active && matched,
				});
			}
			Line::Else => {
				if let Some(frame) = stack.last_mut() {
					frame.active = frame.parent && !frame.matched;
				}
			}
			Line::End => {
				stack.pop();
			}
		}
	}
	Ok(output)
}

fn parse(text: &str) -> Result<Vec<Line<'_>>, BlockError> {
	let mut lines = Vec::new();
	let mut open: Vec<(usize, bool)> = Vec::new();

	for (index, raw) in text.split_inclusive('\n').enumerate() {
		let number = index + 1;
		let Some(marker) = find_directive(raw) else {
			lines.push(Line::Text(raw));
			continue;
		};

		let prefix = raw[..marker].trim_end();
		let opener = OPENERS
			.iter()
			.find(|opener| prefix.ends_with(*opener))
			.ok_or(BlockError::NotInComment(number))?;
		let code = &prefix[..prefix.len() - opener.len()];
		let directive = strip_closer(raw[marker + MARKER.len()..].trim());
		let (word, rest) = directive.split_once(char::is_whitespace).unwrap_or((directive, ""));
		let inline = !code.trim().is_empty();

		let line = match word {
			"if" => {
				let condition = Expr::parse(rest.trim()).map_err(|source| BlockError::Condition { line: number, source })?;
				if inline {
					Line::Inline {
						code: format!("{}{}", code.trim_end(), line_ending(raw)),
						condition,
					}
				} else {
					open.push((number, false));
					if open.len() > MAX_DEPTH {
						return Err(BlockError::TooDeep(number));
					}
					Line::If(condition)
				}
			}
			"else" | "end" if inline => return Err(BlockError::InlineBlock(number)),
			"else" => {
				let frame = open.last_mut().ok_or(BlockError::Unmatched(number))?;
				if frame.1 {
					return Err(BlockError::DoubleElse(number));
				}
				frame.1 = true;
				Line::Else
			}
			"end" => {
				open.pop().ok_or(BlockError::Unmatched(number))?;
				Line::End
			}
			_ => {
				return Err(BlockError::Unknown {
					line: number,
					word: word.to_owned(),
				});
			}
		};
		lines.push(line);
	}

	match open.last() {
		Some(&(line, _)) => Err(BlockError::Unclosed(line)),
		None => Ok(lines),
	}
}

fn find_directive(line: &str) -> Option<usize> {
	line
		.match_indices(MARKER)
		.map(|(index, _)| index)
		.find(|index| line[index + MARKER.len()..].starts_with([' ', '\t']))
}

fn strip_closer(directive: &str) -> &str {
	CLOSERS
		.iter()
		.find_map(|closer| directive.strip_suffix(closer))
		.unwrap_or(directive)
		.trim_end()
}

fn line_ending(line: &str) -> &'static str {
	if line.ends_with("\r\n") {
		"\r\n"
	} else if line.ends_with('\n') {
		"\n"
	} else {
		""
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::Value;

	fn values(styled: bool) -> Values {
		Values::from([("styled".to_owned(), Value::Bool(styled))])
	}

	#[test]
	fn block_should_keep_lines_when_true() {
		let text = "a\n// @tfverse if styled\nb\n// @tfverse end\nc\n";

		assert_eq!(process(text, &values(true)).unwrap(), "a\nb\nc\n");
	}

	#[test]
	fn block_should_drop_lines_when_false() {
		let text = "a\n// @tfverse if styled\nb\n// @tfverse end\nc\n";

		assert_eq!(process(text, &values(false)).unwrap(), "a\nc\n");
	}

	#[test]
	fn else_should_take_the_other_branch() {
		let text = "# @tfverse if styled\nyes\n# @tfverse else\nno\n# @tfverse end\n";

		assert_eq!(process(text, &values(false)).unwrap(), "no\n");
	}

	#[test]
	fn inline_should_strip_the_comment_when_true() {
		let text = "import s from './x.css'; // @tfverse if styled\n";

		assert_eq!(process(text, &values(true)).unwrap(), "import s from './x.css';\n");
	}

	#[test]
	fn inline_should_drop_the_line_when_false() {
		assert_eq!(
			process("keep\nx(); // @tfverse if styled\n", &values(false)).unwrap(),
			"keep\n"
		);
	}

	#[test]
	fn html_comment_should_work() {
		let text = "<!-- @tfverse if styled -->\n<style></style>\n<!-- @tfverse end -->\n";

		assert_eq!(process(text, &values(true)).unwrap(), "<style></style>\n");
	}

	#[test]
	fn jsx_comment_should_work() {
		let text = "{/* @tfverse if styled */}\n<Styled />\n{/* @tfverse end */}\n";

		assert_eq!(process(text, &values(false)).unwrap(), "");
	}

	#[test]
	fn nested_block_should_follow_parent() {
		let text =
			"// @tfverse if styled\n// @tfverse if !styled\nx\n// @tfverse else\ny\n// @tfverse end\n// @tfverse end\n";

		assert_eq!(process(text, &values(false)).unwrap(), "");
	}

	#[test]
	fn crlf_line_endings_should_be_kept() {
		assert_eq!(
			process("x(); // @tfverse if styled\r\n", &values(true)).unwrap(),
			"x();\r\n"
		);
	}

	#[test]
	fn marker_outside_comment_should_fail() {
		assert_eq!(
			process("const a = '@tfverse if x';\n", &values(true)),
			Err(BlockError::NotInComment(1))
		);
	}

	#[test]
	fn unclosed_if_should_fail() {
		assert_eq!(
			process("// @tfverse if styled\nx\n", &values(true)),
			Err(BlockError::Unclosed(1))
		);
	}

	#[test]
	fn stray_end_should_fail() {
		assert_eq!(
			process("// @tfverse end\n", &values(true)),
			Err(BlockError::Unmatched(1))
		);
	}

	#[test]
	fn unknown_directive_should_fail() {
		assert!(matches!(
			process("// @tfverse fi\n", &values(true)),
			Err(BlockError::Unknown { line: 1, .. })
		));
	}

	#[test]
	fn anchors_should_not_count_as_directives() {
		let text = "const routes = [\n  // @tfverse:routes\n];\n";

		assert_eq!(process(text, &values(true)).unwrap(), text);
	}

	#[test]
	fn conditions_should_list_every_condition() {
		let text = "// @tfverse if a\n// @tfverse end\nx(); // @tfverse if b\n";

		assert_eq!(conditions(text).unwrap().len(), 2);
	}
}
