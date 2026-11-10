//! Resolves the values given to a piece against its inputs.

use std::collections::BTreeMap;

use crate::model::{Input, Value, Values, words};

const MAX_TEXT_LEN: usize = 256;

/// Why the given values do not fit the piece's inputs.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InputError {
	/// A value for an input the piece does not have.
	#[error("unknown input `{0}`")]
	Unknown(String),
	/// A required input without a value.
	#[error("input `{0}` is required")]
	Missing(String),
	/// A value of the wrong type.
	#[error("input `{name}` expects {expected}")]
	WrongType {
		/// Input name.
		name: String,
		/// Expected type.
		expected: &'static str,
	},
	/// An enum value outside its choices.
	#[error("input `{name}` must be one of: {choices}")]
	NotAChoice {
		/// Input name.
		name: String,
		/// Allowed values.
		choices: String,
	},
	/// A number outside its bounds.
	#[error("input `{name}` must be between {min} and {max}")]
	OutOfRange {
		/// Input name.
		name: String,
		/// Lower bound.
		min: i64,
		/// Upper bound.
		max: i64,
	},
	/// An identifier with characters other than letters, digits, spaces, `-` and `_`, or the `.` and `/`
	/// its example uses.
	#[error(
		"input `{0}` must be an identifier: letters, digits, spaces, `-` or `_` (and `.` or `/` where its example uses them), starting with a letter"
	)]
	NotAnIdentifier(String),
	/// Text with control characters or over the length limit.
	#[error("input `{0}` must be single-line text up to {MAX_TEXT_LEN} characters")]
	InvalidText(String),
}

/// Checks `given` against `inputs`, filling in defaults.
pub(crate) fn resolve(inputs: &BTreeMap<String, Input>, given: &Values) -> Result<Values, InputError> {
	if let Some(unknown) = given.keys().find(|name| !inputs.contains_key(*name)) {
		return Err(InputError::Unknown(unknown.clone()));
	}

	let mut values = Values::new();
	for (name, input) in inputs {
		let value = match given.get(name) {
			Some(value) => check(name, input, value)?,
			None => default(input).ok_or_else(|| InputError::Missing(name.clone()))?,
		};
		values.insert(name.clone(), value);
	}
	Ok(values)
}

fn default(input: &Input) -> Option<Value> {
	match input {
		Input::String { default, .. } | Input::Enum { default, .. } => default.clone().map(Value::String),
		Input::Bool { default, .. } => Some(Value::Bool(default.unwrap_or(false))),
		Input::Number { default, .. } => default.map(Value::Number),
	}
}

fn check(name: &str, input: &Input, value: &Value) -> Result<Value, InputError> {
	let wrong = |expected| InputError::WrongType {
		name: name.to_owned(),
		expected,
	};
	match (input, value) {
		(Input::String { example, .. }, Value::String(text)) => {
			let identifier = example.as_deref().filter(|example| words(example).len() >= 2);
			if identifier.is_some_and(|example| !is_identifier(text, example)) {
				return Err(InputError::NotAnIdentifier(name.to_owned()));
			}
			if text.chars().count() > MAX_TEXT_LEN || text.chars().any(char::is_control) {
				return Err(InputError::InvalidText(name.to_owned()));
			}
			Ok(value.clone())
		}
		(Input::Bool { .. }, Value::Bool(_)) => Ok(value.clone()),
		(Input::Enum { choices, .. }, Value::String(choice)) => {
			if choices.contains(choice) {
				Ok(value.clone())
			} else {
				Err(InputError::NotAChoice {
					name: name.to_owned(),
					choices: choices.join(", "),
				})
			}
		}
		(Input::Number { min, max, .. }, Value::Number(number)) => {
			let (min, max) = (min.unwrap_or(i64::MIN), max.unwrap_or(i64::MAX));
			if (min..=max).contains(number) {
				Ok(value.clone())
			} else {
				Err(InputError::OutOfRange {
					name: name.to_owned(),
					min,
					max,
				})
			}
		}
		(Input::String { .. }, _) => Err(wrong("text")),
		(Input::Bool { .. }, _) => Err(wrong("true or false")),
		(Input::Enum { .. }, _) => Err(wrong("one of its choices")),
		(Input::Number { .. }, _) => Err(wrong("a whole number")),
	}
}

fn is_identifier(text: &str, example: &str) -> bool {
	let separator = |c: char| matches!(c, '.' | '/') && example.contains(c);
	text.starts_with(|c: char| c.is_ascii_alphabetic())
		&& text
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_') || separator(c))
		&& !text.split(separator).any(str::is_empty)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn inputs() -> BTreeMap<String, Input> {
		BTreeMap::from([
			(
				"name".to_owned(),
				Input::String {
					description: None,
					prompt: None,
					example: Some("MyExample".to_owned()),
					default: None,
				},
			),
			(
				"styled".to_owned(),
				Input::Bool {
					description: None,
					prompt: None,
					default: None,
				},
			),
			(
				"style".to_owned(),
				Input::Enum {
					description: None,
					prompt: None,
					choices: vec!["css".to_owned(), "scss".to_owned()],
					default: Some("css".to_owned()),
				},
			),
		])
	}

	fn given(name: &str) -> Values {
		Values::from([("name".to_owned(), Value::String(name.to_owned()))])
	}

	#[test]
	fn resolve_should_fill_defaults() {
		let values = resolve(&inputs(), &given("User Card")).unwrap();

		assert_eq!(values.get("style"), Some(&Value::String("css".to_owned())));
	}

	#[test]
	fn resolve_should_default_bools_to_false() {
		let values = resolve(&inputs(), &given("Card")).unwrap();

		assert_eq!(values.get("styled"), Some(&Value::Bool(false)));
	}

	#[test]
	fn resolve_should_require_inputs_without_default() {
		assert_eq!(
			resolve(&inputs(), &Values::new()),
			Err(InputError::Missing("name".to_owned()))
		);
	}

	#[test]
	fn resolve_should_reject_unknown_input() {
		let mut values = given("Card");
		values.insert("extra".to_owned(), Value::Bool(true));

		assert_eq!(
			resolve(&inputs(), &values),
			Err(InputError::Unknown("extra".to_owned()))
		);
	}

	#[test]
	fn resolve_should_reject_path_like_identifier() {
		assert_eq!(
			resolve(&inputs(), &given("../evil")),
			Err(InputError::NotAnIdentifier("name".to_owned()))
		);
	}

	fn module_input() -> BTreeMap<String, Input> {
		BTreeMap::from([(
			"module".to_owned(),
			Input::String {
				description: None,
				prompt: None,
				example: Some("example.com/app".to_owned()),
				default: None,
			},
		)])
	}

	fn module(value: &str) -> Result<Values, InputError> {
		resolve(
			&module_input(),
			&Values::from([("module".to_owned(), Value::String(value.to_owned()))]),
		)
	}

	#[test]
	fn resolve_should_allow_separators_the_example_uses() {
		assert!(module("github.com/acme/shop").is_ok());
		assert!(resolve(&inputs(), &given("acme.card")).is_err());
	}

	#[test]
	fn resolve_should_reject_empty_segments_between_separators() {
		for value in ["github.com//shop", "github.com/../etc", "github.com/shop/"] {
			assert_eq!(
				module(value),
				Err(InputError::NotAnIdentifier("module".to_owned())),
				"{value}"
			);
		}
	}

	#[test]
	fn resolve_should_reject_value_outside_choices() {
		let mut values = given("Card");
		values.insert("style".to_owned(), Value::String("less".to_owned()));

		assert!(matches!(
			resolve(&inputs(), &values),
			Err(InputError::NotAChoice { .. })
		));
	}
}
