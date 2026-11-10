//! Piece inputs from the command line, and prompts for the ones still missing.

use anyhow::{anyhow, bail};
use tfverse_core::model::{Input, PieceManifest};
use tfverse_core::{Value, Values};

use crate::ui::ask;

/// Values from the positional name and `key=value` pairs, typed by what the piece declares.
pub fn parse(manifest: &PieceManifest, name: Option<&str>, pairs: &[String]) -> anyhow::Result<Values> {
	let mut values = Values::new();
	if let Some(name) = name {
		if !manifest.inputs.contains_key("name") {
			bail!(
				"piece `{}` has no `name` input; pass inputs with -i key=value",
				manifest.name
			);
		}
		values.insert("name".to_owned(), Value::String(name.to_owned()));
	}
	for pair in pairs {
		let (key, text) = pair
			.split_once('=')
			.ok_or_else(|| anyhow!("expected key=value, found `{pair}`"))?;
		let value = match manifest.inputs.get(key) {
			Some(Input::Bool { .. }) => Value::Bool(match text.to_ascii_lowercase().as_str() {
				"true" | "yes" | "y" | "1" | "on" => true,
				"false" | "no" | "n" | "0" | "off" => false,
				_ => bail!("`{key}` is true or false, not `{text}`"),
			}),
			Some(Input::Number { .. }) => Value::Number(
				text
					.parse()
					.map_err(|_| anyhow!("`{key}` is a whole number, not `{text}`"))?,
			),
			_ => Value::String(text.to_owned()),
		};
		values.insert(key.to_owned(), value);
	}
	Ok(values)
}

/// Asks on the terminal for every input that has no value and no default.
pub fn prompt_missing(manifest: &PieceManifest, values: &mut Values) -> anyhow::Result<()> {
	for (key, input) in &manifest.inputs {
		if values.contains_key(key) {
			continue;
		}
		let (question, has_default, choices) = match input {
			Input::String {
				prompt,
				description,
				default,
				..
			} => (prompt.clone().or_else(|| description.clone()), default.is_some(), None),
			Input::Enum {
				prompt,
				description,
				default,
				choices,
				..
			} => (
				prompt.clone().or_else(|| description.clone()),
				default.is_some(),
				Some(choices.clone()),
			),
			Input::Number {
				prompt,
				description,
				default,
				..
			} => (prompt.clone().or_else(|| description.clone()), default.is_some(), None),
			Input::Bool { .. } => continue,
		};
		if has_default {
			continue;
		}
		let label = question.map_or_else(|| key.clone(), |question| question.trim_end_matches('.').to_owned());
		let hint = choices
			.as_ref()
			.map(|choices| format!(" ({})", choices.join(", ")))
			.unwrap_or_default();
		let answer = ask(&format!("{label}{hint}: "))?;
		if answer.is_empty() {
			bail!("`{key}` is required");
		}
		let pair = format!("{key}={answer}");
		values.extend(parse(manifest, None, &[pair])?);
	}
	Ok(())
}
