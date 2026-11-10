//! Replaces example identifiers with input values in every case form.

use std::borrow::Cow;

use crate::model::{Case, Value, words};

/// Why the examples cannot be used together.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplaceError {
	/// Two inputs produce the same text to replace.
	#[error("examples of `{first}` and `{second}` both produce `{pattern}`")]
	Ambiguous {
		/// First input.
		first: String,
		/// Second input.
		second: String,
		/// The shared text.
		pattern: String,
	},
}

/// One input's example and the value replacing it.
#[derive(Clone, Debug)]
pub struct Example<'a> {
	/// Input name.
	pub input: &'a str,
	/// The example as written in templates.
	pub text: Cow<'a, str>,
	/// The value given by the user.
	pub value: &'a Value,
}

/// Replaces every example in a single left-to-right pass, longest match first.
#[derive(Debug, Default)]
pub struct Replacer {
	rules: Vec<Rule>,
}

#[derive(Debug)]
struct Rule {
	input: String,
	pattern: String,
	replacement: String,
}

impl Replacer {
	/// Builds the replacement rules.
	///
	/// An example of two or more words is replaced in every [`Case`] and as written, e.g.
	/// `com.example.app`; any other example is replaced literally.
	pub fn new(examples: &[Example<'_>]) -> Result<Self, ReplaceError> {
		let mut rules: Vec<Rule> = Vec::new();
		for example in examples {
			for (pattern, replacement) in expand(example) {
				if let Some(existing) = rules.iter().find(|rule| rule.pattern == pattern) {
					if existing.input != example.input {
						return Err(ReplaceError::Ambiguous {
							first: existing.input.clone(),
							second: example.input.to_owned(),
							pattern,
						});
					}
					continue;
				}
				rules.push(Rule {
					input: example.input.to_owned(),
					pattern,
					replacement,
				});
			}
		}
		rules.sort_by(|a, b| {
			b.pattern
				.len()
				.cmp(&a.pattern.len())
				.then_with(|| a.pattern.cmp(&b.pattern))
		});
		Ok(Self { rules })
	}

	/// Applies every rule to `text`.
	#[must_use]
	pub fn apply(&self, text: &str) -> String {
		if self.rules.is_empty() {
			return text.to_owned();
		}
		let mut output = String::with_capacity(text.len());
		let mut rest = text;
		while let Some(c) = rest.chars().next() {
			if let Some(rule) = self.rules.iter().find(|rule| rest.starts_with(&rule.pattern)) {
				output.push_str(&rule.replacement);
				rest = &rest[rule.pattern.len()..];
			} else {
				output.push(c);
				rest = &rest[c.len_utf8()..];
			}
		}
		output
	}
}

fn expand(example: &Example<'_>) -> Vec<(String, String)> {
	let example_words = words(&example.text);
	match example.value {
		Value::String(value) if example_words.len() >= 2 => {
			let value_words = words(value);
			let mut rules: Vec<_> = Case::ALL
				.iter()
				.map(|case| (case.join(&example_words), case.join(&value_words)))
				.collect();
			if rules.iter().all(|(pattern, _)| *pattern != example.text) {
				rules.push((example.text.clone().into_owned(), value.clone()));
			}
			rules
		}
		value => vec![(example.text.clone().into_owned(), value.to_string())],
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn replacer(example: &str, value: &str) -> Replacer {
		let value = Value::String(value.to_owned());
		Replacer::new(&[Example {
			input: "name",
			text: example.into(),
			value: &value,
		}])
		.unwrap()
	}

	#[test]
	fn apply_should_replace_every_case_form() {
		let text = "MyExample myExample my-example my_example MY_EXAMPLE";

		assert_eq!(
			replacer("MyExample", "UserProfile").apply(text),
			"UserProfile userProfile user-profile user_profile USER_PROFILE"
		);
	}

	#[test]
	fn apply_should_replace_dotted_example_as_written_and_as_path() {
		let text = "package com.example.app\nsrc/com/example/app";

		assert_eq!(
			replacer("com.example.app", "dev.acme.shop").apply(text),
			"package dev.acme.shop\nsrc/dev/acme/shop"
		);
	}

	#[test]
	fn apply_should_replace_module_path_as_written() {
		assert_eq!(
			replacer("example.com/app", "github.com/acme/shop").apply("import \"example.com/app/internal\""),
			"import \"github.com/acme/shop/internal\""
		);
	}

	#[test]
	fn apply_should_replace_inside_longer_identifiers() {
		assert_eq!(
			replacer("MyExample", "Card").apply("useMyExample(MyExamples)"),
			"useCard(Cards)"
		);
	}

	#[test]
	fn single_word_example_should_replace_literally() {
		assert_eq!(replacer("3000", "8080").apply("port 3000"), "port 8080");
	}

	#[test]
	fn single_word_example_should_not_change_other_cases() {
		assert_eq!(replacer("Example", "Card").apply("Example example"), "Card example");
	}

	#[test]
	fn apply_should_not_rescan_replaced_text() {
		assert_eq!(
			replacer("MyExample", "MyExampleList").apply("MyExample"),
			"MyExampleList"
		);
	}

	#[test]
	fn new_should_reject_examples_with_shared_patterns() {
		let value = Value::String("x".to_owned());
		let examples = [
			Example {
				input: "a",
				text: "MyExample".into(),
				value: &value,
			},
			Example {
				input: "b",
				text: "my-example".into(),
				value: &value,
			},
		];

		assert!(matches!(Replacer::new(&examples), Err(ReplaceError::Ambiguous { .. })));
	}
}
