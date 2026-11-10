//! JSON Schemas for every file format, generated from the Rust types.

use schemars::schema_for;

use crate::model::{PackManifest, PieceManifest};
use crate::workspace::{Lockfile, ProjectConfig, UserConfig};

/// Every schema, as `(file name, pretty JSON)`.
#[must_use]
pub fn all() -> Vec<(&'static str, String)> {
	[
		("pack.schema.json", serde_json::to_value(schema_for!(PackManifest))),
		("piece.schema.json", serde_json::to_value(schema_for!(PieceManifest))),
		("lock.schema.json", serde_json::to_value(schema_for!(Lockfile))),
		("config.schema.json", serde_json::to_value(schema_for!(ProjectConfig))),
		("user-config.schema.json", serde_json::to_value(schema_for!(UserConfig))),
	]
	.into_iter()
	.map(|(name, value)| {
		let text = value
			.and_then(|value| serde_json::to_string_pretty(&value))
			.unwrap_or_default();
		(name, format!("{text}\n"))
	})
	.collect()
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;

	#[test]
	fn committed_schemas_should_match_the_types() {
		let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
		let update = std::env::var_os("TFVERSE_UPDATE_SCHEMAS").is_some();
		let mut stale = Vec::new();

		for (name, text) in all() {
			let path = dir.join(name);
			if update {
				std::fs::create_dir_all(&dir).unwrap();
				std::fs::write(&path, &text).unwrap();
			} else if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
				stale.push(name);
			}
		}

		assert!(
			stale.is_empty(),
			"schemas out of date: {stale:?}; run `cargo xtask schema`"
		);
	}
}
