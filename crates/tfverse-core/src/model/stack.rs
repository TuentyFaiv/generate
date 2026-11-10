//! Facts about a project's stack, used to pick variants and in conditions.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::{Fact, Value, Values};

/// Detected or configured stack facts, e.g. `framework = {react, next}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Stack(BTreeMap<Fact, BTreeSet<String>>);

impl Stack {
	/// Whether no fact is known.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.0.values().all(BTreeSet::is_empty)
	}

	/// Adds one fact value.
	pub fn insert(&mut self, fact: Fact, value: &str) {
		self.0.entry(fact).or_default().insert(value.to_owned());
	}

	/// Replaces every value of `fact`.
	pub fn set(&mut self, fact: Fact, values: impl IntoIterator<Item = String>) {
		self.0.insert(fact, values.into_iter().collect());
	}

	/// Whether `fact` includes `value`.
	#[must_use]
	pub fn has(&self, fact: Fact, value: &str) -> bool {
		self.0.get(&fact).is_some_and(|values| values.contains(value))
	}

	/// Values of `fact`.
	pub fn values(&self, fact: Fact) -> impl Iterator<Item = &str> {
		self.0.get(&fact).into_iter().flatten().map(String::as_str)
	}

	/// Whether every requirement holds.
	#[must_use]
	pub fn satisfies(&self, requirements: &BTreeMap<Fact, String>) -> bool {
		requirements.iter().all(|(fact, value)| self.has(*fact, value))
	}

	/// Condition values: `stack.<value>` is true for every known fact value.
	#[must_use]
	pub fn condition_values(&self) -> Values {
		self
			.0
			.values()
			.flatten()
			.map(|value| (format!("stack.{value}"), Value::Bool(true)))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn satisfies_should_require_every_fact() {
		let mut stack = Stack::default();
		stack.insert(Fact::Framework, "react");

		let wants = BTreeMap::from([
			(Fact::Framework, "react".to_owned()),
			(Fact::Language, "typescript".to_owned()),
		]);

		assert!(!stack.satisfies(&wants));
	}

	#[test]
	fn condition_values_should_expose_stack_prefix() {
		let mut stack = Stack::default();
		stack.insert(Fact::Styling, "tailwind");

		assert_eq!(stack.condition_values().get("stack.tailwind"), Some(&Value::Bool(true)));
	}
}
