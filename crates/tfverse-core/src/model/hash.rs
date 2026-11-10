//! SHA-256 content digests.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

const PREFIX: &str = "sha256:";

/// SHA-256 digest of some content, written as `sha256:<hex>`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest([u8; 32]);

impl Digest {
	/// Hashes `bytes`.
	#[must_use]
	pub fn of(bytes: &[u8]) -> Self {
		Self(Sha256::digest(bytes).into())
	}

	/// Hashes a set of named files; the same files always give the same digest.
	#[must_use]
	pub fn of_files<'a>(files: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> Self {
		let mut hasher = Sha256::new();
		for (path, bytes) in files {
			for part in [path.as_bytes(), bytes] {
				hasher.update((part.len() as u64).to_le_bytes());
				hasher.update(part);
			}
		}
		Self(hasher.finalize().into())
	}

	/// Lowercase hex, without the `sha256:` prefix.
	#[must_use]
	pub fn hex(&self) -> String {
		use fmt::Write as _;

		self.0.iter().fold(String::with_capacity(64), |mut hex, byte| {
			let _ = write!(hex, "{byte:02x}");
			hex
		})
	}
}

impl fmt::Display for Digest {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{PREFIX}{}", self.hex())
	}
}

impl fmt::Debug for Digest {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

/// Why a digest string was rejected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("`{0}` is not a `sha256:` digest with 64 lowercase hex characters")]
pub struct ParseDigestError(String);

impl FromStr for Digest {
	type Err = ParseDigestError;

	fn from_str(text: &str) -> Result<Self, Self::Err> {
		let invalid = || ParseDigestError(text.to_owned());
		let hex = text.strip_prefix(PREFIX).ok_or_else(invalid)?;
		if hex.len() != 64 {
			return Err(invalid());
		}

		let mut bytes = [0; 32];
		for (byte, pair) in bytes.iter_mut().zip(hex.as_bytes().as_chunks::<2>().0) {
			let high = hex_value(pair[0]).ok_or_else(invalid)?;
			let low = hex_value(pair[1]).ok_or_else(invalid)?;
			*byte = high << 4 | low;
		}
		Ok(Self(bytes))
	}
}

fn hex_value(digit: u8) -> Option<u8> {
	match digit {
		b'0'..=b'9' => Some(digit - b'0'),
		b'a'..=b'f' => Some(digit - b'a' + 10),
		_ => None,
	}
}

impl Serialize for Digest {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.collect_str(self)
	}
}

impl<'de> Deserialize<'de> for Digest {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let text = Cow::<str>::deserialize(deserializer)?;
		text.parse().map_err(serde::de::Error::custom)
	}
}

impl JsonSchema for Digest {
	fn schema_name() -> Cow<'static, str> {
		"Digest".into()
	}

	fn json_schema(_: &mut SchemaGenerator) -> Schema {
		json_schema!({
			"type": "string",
			"pattern": "^sha256:[0-9a-f]{64}$"
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn of_should_match_known_sha256() {
		assert_eq!(
			Digest::of(b"abc").to_string(),
			"sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
		);
	}

	#[test]
	fn parse_should_round_trip_display() {
		let digest = Digest::of(b"tfverse");

		assert_eq!(digest.to_string().parse::<Digest>(), Ok(digest));
	}

	#[test]
	fn parse_should_reject_uppercase_hex() {
		let upper = Digest::of(b"x")
			.to_string()
			.to_uppercase()
			.replace("SHA256:", "sha256:");

		assert!(upper.parse::<Digest>().is_err());
	}

	#[test]
	fn parse_should_reject_missing_prefix() {
		assert!(Digest::of(b"x").hex().parse::<Digest>().is_err());
	}
}
