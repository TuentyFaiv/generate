//! Input values.

use std::collections::BTreeMap;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A value given to a piece input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Value {
	/// `true` or `false`.
	Bool(bool),
	/// A whole number.
	Number(i64),
	/// Text.
	String(String),
}

impl Value {
	/// Whether the value counts as true in a condition.
	#[must_use]
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Bool(value) => *value,
			Self::Number(value) => *value != 0,
			Self::String(value) => !value.is_empty(),
		}
	}
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Bool(value) => write!(f, "{value}"),
			Self::Number(value) => write!(f, "{value}"),
			Self::String(value) => f.write_str(value),
		}
	}
}

/// Input values by name.
pub type Values = BTreeMap<String, Value>;
