use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

const MAX_NAME_LEN: usize = 120;

/// An example item as stored and returned by the API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[derive(sqlx::FromRow)] // @tfverse if stack.sqlx
#[serde(rename_all = "camelCase")]
pub struct ExampleItem {
	pub id: Uuid,
	pub name: String,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

/// Fields accepted when creating an example item.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewExampleItem {
	pub name: String,
}

/// Fields accepted when changing an example item; missing ones stay as they are.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleItemChanges {
	pub name: Option<String>,
}

/// A trimmed name of 1 to 120 characters.
pub(super) fn valid_name(name: &str) -> Result<String, AppError> {
	let name = name.trim();
	if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
		return Err(AppError::Validation(format!("name must be 1 to {MAX_NAME_LEN} characters")));
	}
	Ok(name.to_owned())
}
