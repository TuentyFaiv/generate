//! Adds a JS/TS import after the existing ones, including Svelte, Vue and Astro script blocks.

use std::ops::Range;

use crate::patch::Patch;
use crate::patch::{PatchError, Patched, Patcher, line_ending};

/// The built-in import patcher.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImportPatcher;

impl Patcher for ImportPatcher {
	fn apply(&self, text: &str, patch: &Patch) -> Result<Patched, PatchError> {
		let Patch::Import { import, .. } = patch else {
			return Err(PatchError::Invalid(
				"import patcher received another patch kind".to_owned(),
			));
		};
		let import = import.trim();
		if !import.starts_with("import") {
			return Err(PatchError::Invalid(format!("`{import}` is not an import statement")));
		}

		let region = script_region(patch.file().as_str(), text)?;
		let code = &text[region.clone()];
		let scan = scan(code);
		let ending = line_ending(text);
		let inserted = format!("{import}{ending}");

		let wanted = normalize(import);
		if scan
			.imports
			.iter()
			.any(|range| normalize(&code[range.clone()]) == wanted)
		{
			return Ok(Patched::AlreadyApplied { inserted });
		}

		let at = match scan.imports.last() {
			Some(last) => after_line(code, last.end),
			None => scan.prologue_end,
		};
		let offset = region.start + at;
		let mut output = String::with_capacity(text.len() + inserted.len() + 1);
		output.push_str(&text[..offset]);
		if offset > 0 && !text[..offset].ends_with('\n') {
			output.push_str(ending);
		}
		output.push_str(&inserted);
		output.push_str(&text[offset..]);
		Ok(Patched::Changed { text: output, inserted })
	}
}

fn script_region(file: &str, text: &str) -> Result<Range<usize>, PatchError> {
	let extension = file.rsplit_once('.').map_or("", |(_, extension)| extension);
	match extension {
		"svelte" | "vue" => {
			script_block(text).ok_or_else(|| PatchError::Invalid(format!("`{file}` has no <script> block")))
		}
		"astro" => frontmatter(text).ok_or_else(|| PatchError::Invalid(format!("`{file}` has no `---` frontmatter"))),
		_ => Ok(0..text.len()),
	}
}

fn script_block(text: &str) -> Option<Range<usize>> {
	let mut search = 0;
	let mut fallback = None;
	while let Some(found) = text[search..].find("<script") {
		let tag_start = search + found;
		let tag_end = tag_start + text[tag_start..].find('>')? + 1;
		let body_end = tag_end + text[tag_end..].find("</script>")?;
		let tag = &text[tag_start..tag_end];
		let range = tag_end..body_end;
		if !tag.contains("module") {
			return Some(range);
		}
		fallback.get_or_insert(range);
		search = body_end;
	}
	fallback
}

fn frontmatter(text: &str) -> Option<Range<usize>> {
	let rest = text.strip_prefix("---")?;
	let start = 3 + rest.find('\n')? + 1;
	let mut offset = start;
	for line in text[start..].split_inclusive('\n') {
		if line.trim_end() == "---" {
			return Some(start..offset);
		}
		offset += line.len();
	}
	None
}

struct Scan {
	imports: Vec<Range<usize>>,
	prologue_end: usize,
}

fn scan(code: &str) -> Scan {
	let bytes = code.as_bytes();
	let mut i = 0;
	if code.starts_with("#!") {
		i = after_line(code, 0);
	}
	let mut prologue_end = i;
	let mut imports = Vec::new();

	loop {
		i = skip_trivia(bytes, i);
		if i >= bytes.len() {
			break;
		}
		if imports.is_empty() && matches!(bytes[i], b'"' | b'\'') {
			i = skip_string(bytes, i);
			i = skip_inline_space(bytes, i);
			if bytes.get(i) == Some(&b';') {
				i += 1;
			}
			prologue_end = after_line(code, i);
			continue;
		}
		if is_import_statement(bytes, i) {
			let end = import_end(bytes, i);
			imports.push(i..end);
			i = end;
			continue;
		}
		break;
	}
	if imports.is_empty() {
		prologue_end = skip_leading_comments(code, prologue_end);
	}
	Scan { imports, prologue_end }
}

fn is_import_statement(bytes: &[u8], i: usize) -> bool {
	if !bytes[i..].starts_with(b"import") || bytes.get(i + 6).is_some_and(|&b| is_ident(b)) {
		return false;
	}
	let next = skip_trivia(bytes, i + 6);
	!matches!(bytes.get(next), Some(b'(' | b'.'))
}

fn import_end(bytes: &[u8], start: usize) -> usize {
	let mut i = start + 6;
	loop {
		i = skip_trivia(bytes, i);
		match bytes.get(i) {
			None => return bytes.len(),
			Some(b'{') => i = skip_braces(bytes, i),
			Some(b'"' | b'\'') => {
				i = skip_string(bytes, i);
				let mut end = i;
				let next = skip_trivia(bytes, i);
				if bytes[next..].starts_with(b"with") || bytes[next..].starts_with(b"assert") {
					let brace = skip_trivia(bytes, next + if bytes[next..].starts_with(b"with") { 4 } else { 6 });
					if bytes.get(brace) == Some(&b'{') {
						end = skip_braces(bytes, brace);
					}
				}
				let after = skip_inline_space(bytes, end);
				return if bytes.get(after) == Some(&b';') {
					after + 1
				} else {
					end
				};
			}
			Some(b';') => return i + 1,
			Some(_) => i += 1,
		}
	}
}

fn skip_trivia(bytes: &[u8], mut i: usize) -> usize {
	loop {
		while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
			i += 1;
		}
		if bytes[i..].starts_with(b"//") {
			i = bytes[i..]
				.iter()
				.position(|&b| b == b'\n')
				.map_or(bytes.len(), |p| i + p + 1);
		} else if bytes[i..].starts_with(b"/*") {
			i = find(bytes, i + 2, b"*/").map_or(bytes.len(), |p| p + 2);
		} else {
			return i;
		}
	}
}

fn skip_inline_space(bytes: &[u8], mut i: usize) -> usize {
	while matches!(bytes.get(i), Some(b' ' | b'\t')) {
		i += 1;
	}
	i
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
	let quote = bytes[start];
	let mut i = start + 1;
	while let Some(&b) = bytes.get(i) {
		i += 1;
		if b == b'\\' {
			i += 1;
		} else if b == quote || (b == b'\n' && quote != b'`') {
			return i;
		}
	}
	bytes.len()
}

fn skip_braces(bytes: &[u8], start: usize) -> usize {
	let mut depth = 0;
	let mut i = start;
	while i < bytes.len() {
		match bytes[i] {
			b'{' => depth += 1,
			b'}' => {
				depth -= 1;
				if depth == 0 {
					return i + 1;
				}
			}
			b'"' | b'\'' | b'`' => {
				i = skip_string(bytes, i);
				continue;
			}
			b'/' if bytes.get(i + 1).is_some_and(|&b| b == b'/' || b == b'*') => {
				i = skip_trivia(bytes, i);
				continue;
			}
			_ => {}
		}
		i += 1;
	}
	bytes.len()
}

fn skip_leading_comments(code: &str, start: usize) -> usize {
	let bytes = code.as_bytes();
	let mut i = start;
	loop {
		let next = skip_inline_space(bytes, i);
		if bytes[next..].starts_with(b"//") || bytes[next..].starts_with(b"/*") {
			let end = skip_trivia(bytes, next);
			i = if bytes[..end].ends_with(b"\n") {
				end
			} else {
				after_line(code, end)
			};
		} else {
			return i;
		}
	}
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
	bytes[from..]
		.windows(needle.len())
		.position(|window| window == needle)
		.map(|p| from + p)
}

fn after_line(code: &str, from: usize) -> usize {
	code[from..].find('\n').map_or(code.len(), |p| from + p + 1)
}

fn is_ident(b: u8) -> bool {
	b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

fn normalize(statement: &str) -> String {
	statement
		.trim()
		.trim_end_matches(';')
		.split_whitespace()
		.collect::<Vec<_>>()
		.join(" ")
		.replace('"', "'")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::RelPath;

	fn add(file: &'static str, text: &str, import: &str) -> Result<Patched, PatchError> {
		let request = Patch::Import {
			file: RelPath::constant(file),
			import: import.to_owned(),
		};
		ImportPatcher.apply(text, &request)
	}

	fn changed(file: &'static str, text: &str, import: &str) -> String {
		match add(file, text, import) {
			Ok(Patched::Changed { text, .. }) => text,
			other => panic!("expected a change, got {other:?}"),
		}
	}

	#[test]
	fn should_append_after_last_import() {
		let text = "import a from 'a';\nimport { b } from \"b\";\n\nconsole.log(a, b);\n";

		assert_eq!(
			changed("src/index.ts", text, "import { Card } from './Card';"),
			"import a from 'a';\nimport { b } from \"b\";\nimport { Card } from './Card';\n\nconsole.log(a, b);\n"
		);
	}

	#[test]
	fn should_handle_multiline_named_imports() {
		let text = "import {\n  a,\n  b,\n} from 'x';\nrun();\n";

		assert_eq!(
			changed("a.ts", text, "import c from 'c';"),
			"import {\n  a,\n  b,\n} from 'x';\nimport c from 'c';\nrun();\n"
		);
	}

	#[test]
	fn should_ignore_imports_inside_strings_and_comments() {
		let text = "// import fake from 'fake';\nconst s = \"import x from 'y'\";\n";

		assert_eq!(
			changed("a.ts", text, "import c from 'c';"),
			"// import fake from 'fake';\nimport c from 'c';\nconst s = \"import x from 'y'\";\n"
		);
	}

	#[test]
	fn should_keep_directives_first() {
		let text = "'use client';\n\nexport const x = 1;\n";

		assert_eq!(
			changed("a.tsx", text, "import c from 'c';"),
			"'use client';\nimport c from 'c';\n\nexport const x = 1;\n"
		);
	}

	#[test]
	fn should_not_treat_dynamic_import_as_statement() {
		let text = "import a from 'a';\nimport('lazy');\n";

		assert_eq!(
			changed("a.ts", text, "import c from 'c';"),
			"import a from 'a';\nimport c from 'c';\nimport('lazy');\n"
		);
	}

	#[test]
	fn should_skip_equivalent_import() {
		let text = "import { Card } from \"./Card\"\n";

		assert!(matches!(
			add("a.ts", text, "import { Card } from './Card';"),
			Ok(Patched::AlreadyApplied { .. })
		));
	}

	#[test]
	fn should_patch_svelte_instance_script() {
		let text =
			"<script module>\nexport const m = 1;\n</script>\n<script lang=\"ts\">\n  import a from 'a';\n</script>\n<p />\n";

		assert_eq!(
			changed("Page.svelte", text, "import c from 'c';"),
			"<script module>\nexport const m = 1;\n</script>\n<script lang=\"ts\">\n  import a from 'a';\nimport c from 'c';\n</script>\n<p />\n"
		);
	}

	#[test]
	fn should_patch_astro_frontmatter() {
		let text = "---\nimport Layout from '../Layout.astro';\n---\n<Layout />\n";

		assert_eq!(
			changed("index.astro", text, "import Card from '../Card.astro';"),
			"---\nimport Layout from '../Layout.astro';\nimport Card from '../Card.astro';\n---\n<Layout />\n"
		);
	}

	#[test]
	fn should_insert_at_top_of_empty_module() {
		assert_eq!(
			changed("a.ts", "export {};\n", "import c from 'c';"),
			"import c from 'c';\nexport {};\n"
		);
	}

	#[test]
	fn should_reject_non_import_text() {
		assert!(matches!(add("a.ts", "", "console.log(1)"), Err(PatchError::Invalid(_))));
	}
}
