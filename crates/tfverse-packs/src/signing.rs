//! Minisign signatures on packs: which keys to trust, which are revoked, and whether to require one.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use minisign_verify::{PublicKey, Signature};
use tfverse_core::Pack;
use tfverse_core::workspace::SigningConfig;

use crate::SourceError;

/// Key ids revoked by the tfverse project, refused for every user.
const REVOKED: &str = include_str!("../revoked-keys.txt");

/// Trusted keys and revocations from the user config.
#[derive(Clone, Debug, Default)]
pub struct Trust {
	keys: BTreeMap<String, (String, PublicKey)>,
	revoked: BTreeSet<String>,
	require: bool,
}

impl Trust {
	/// Trust from the user config's `signing` table plus the built-in revocation list.
	pub fn new(config: &SigningConfig) -> Result<Self, SourceError> {
		let mut keys = BTreeMap::new();
		for (name, encoded) in &config.keys {
			let invalid = || SourceError::Signature(format!("`signing.keys.{name}` is not a minisign public key"));
			let key = PublicKey::from_base64(encoded.trim()).map_err(|_| invalid())?;
			let id = key_id(encoded.trim(), 32).ok_or_else(invalid)?;
			keys.insert(id, (name.clone(), key));
		}
		let mut revoked: BTreeSet<String> = REVOKED
			.lines()
			.map(str::trim)
			.filter(|line| !line.is_empty() && !line.starts_with('#'))
			.map(str::to_ascii_uppercase)
			.collect();
		revoked.extend(config.revoked.iter().map(|id| id.trim().to_ascii_uppercase()));
		Ok(Self {
			keys,
			revoked,
			require: config.require,
		})
	}

	/// Checks `pack`'s signature; returns the pack with its signer when a trusted key signed it.
	///
	/// A bad signature or a revoked key is always refused; an unsigned pack, or one signed by a key
	/// you have not trusted, is refused only when signatures are required.
	pub fn verify(&self, pack: Pack) -> Result<Pack, SourceError> {
		let name = pack.manifest().name.to_string();
		let Some(text) = pack.signature() else {
			return if self.require {
				Err(SourceError::Signature(format!(
					"pack `{name}` is not signed, and your user config requires signed packs"
				)))
			} else {
				Ok(pack)
			};
		};
		let refuse = |reason: String| SourceError::Signature(format!("pack `{name}`: {reason}"));
		let signature =
			Signature::decode(text).map_err(|_| refuse("`pack.minisig` is not a minisign signature".to_owned()))?;
		let id = signature_key_id(text).ok_or_else(|| refuse("`pack.minisig` is not a minisign signature".to_owned()))?;
		if self.revoked.contains(&id) {
			return Err(refuse(format!("signed by key {id}, which is revoked")));
		}
		let Some((_, key)) = self.keys.get(&id) else {
			return if self.require {
				Err(refuse(format!(
					"signed by key {id}, which you have not trusted; add it to `signing.keys` in your user config"
				)))
			} else {
				Ok(pack)
			};
		};
		key
			.verify(pack.signed_message().as_bytes(), &signature, false)
			.map_err(|_| refuse(format!("the signature by key {id} does not match its files")))?;
		Ok(pack.with_signer(id))
	}
}

/// The key id in a signature file: the second line holds the algorithm, the id and the signature.
fn signature_key_id(text: &str) -> Option<String> {
	key_id(text.lines().nth(1)?.trim(), 64)
}

/// The key id of a minisign key or signature line, as minisign prints it: 16 uppercase hex digits.
fn key_id(encoded: &str, rest: usize) -> Option<String> {
	let bytes = base64(encoded)?;
	if bytes.len() != 2 + 8 + rest {
		return None;
	}
	Some(
		bytes[2..10]
			.iter()
			.rev()
			.fold(String::with_capacity(16), |mut id, byte| {
				let _ = write!(id, "{byte:02X}");
				id
			}),
	)
}

/// Standard base64 with padding.
fn base64(text: &str) -> Option<Vec<u8>> {
	let value = |c: u8| match c {
		b'A'..=b'Z' => Some(c - b'A'),
		b'a'..=b'z' => Some(c - b'a' + 26),
		b'0'..=b'9' => Some(c - b'0' + 52),
		b'+' => Some(62),
		b'/' => Some(63),
		_ => None,
	};
	let text = text.as_bytes();
	if !text.len().is_multiple_of(4) {
		return None;
	}
	let mut bytes = Vec::with_capacity(text.len() / 4 * 3);
	for chunk in text.chunks(4) {
		let padding = chunk.iter().rev().take_while(|c| **c == b'=').count();
		let mut group = 0_u32;
		for (index, c) in chunk.iter().enumerate() {
			let digit = if index >= 4 - padding { 0 } else { value(*c)? };
			group = (group << 6) | u32::from(digit);
		}
		let decoded = group.to_be_bytes();
		bytes.extend_from_slice(&decoded[1..4 - padding]);
	}
	Some(bytes)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;

	pub(crate) const TRUSTED_KEY: &str = "RWSRkedr43o7HEk73jaCXj6+QJJ9K+4WxUI/odF8SU0H/wqZ/uT3+r83";
	pub(crate) const TRUSTED_SIG: &str = "untrusted comment: signature from rsign secret key\nRUSRkedr43o7HEJbk6GySzueuaV7sy16DUtad22i9/zDSAiYdYRp7CajRpjCOTAyGlklCHVjmx6ADDXAUlikox1yUlahVom/pwE=\ntrusted comment: tfverse test\naXoDv80yVTWFPizLUm+WuLl4c6qs8MHGyBFrNpmYLS0xScoN0bPlIsMi/hJkvtL9xeRTytByaWi/h1LQstBkBg==\n";
	pub(crate) const OTHER_SIG: &str = "untrusted comment: signature from rsign secret key\nRUQ4CLriJlzPneqFrl8AZ5J6pIdZ5VXZ2phbc+01BTiRmlMODTm+ztbz8u9HtA6VfpEXXVkMypAdurls4xet/OlERydjrUxBCg0=\ntrusted comment: tfverse test\nD2OhmpsTA5nB66d4LRzmPtihTvJY+eXgrpFkgHWaeCNeiyAk5T9R/3GRTuPRPN6uqM0WIcv1imOHDLq8dF0OAA==\n";

	const PACK: &str = "schema_version = 1\nname = \"ui\"\nversion = \"1.0.0\"\n";
	const PIECE: &str = "schema_version = 1\nname = \"atom\"\ntarget = \"src\"\n";

	/// The pack the test signatures were made for, with `atom` as its one template and `signature`.
	fn pack(atom: &str, signature: Option<&str>) -> Pack {
		let mut files = BTreeMap::from([
			("pack.toml".to_owned(), PACK.as_bytes().to_vec()),
			("pieces/atom/piece.toml".to_owned(), PIECE.as_bytes().to_vec()),
			("pieces/atom/files/atom.ts".to_owned(), atom.as_bytes().to_vec()),
		]);
		if let Some(signature) = signature {
			files.insert("pack.minisig".to_owned(), signature.as_bytes().to_vec());
		}
		Pack::from_files(files).unwrap()
	}

	fn trust(require: bool, revoked: &[&str]) -> Trust {
		Trust::new(&SigningConfig {
			require,
			keys: BTreeMap::from([("acme".to_owned(), TRUSTED_KEY.to_owned())]),
			revoked: revoked.iter().map(|id| (*id).to_owned()).collect(),
		})
		.unwrap()
	}

	fn trusted_id() -> String {
		key_id(TRUSTED_KEY, 32).unwrap()
	}

	#[test]
	fn verify_should_record_the_signer_of_a_valid_signature() {
		let pack = trust(true, &[]).verify(pack("x\n", Some(TRUSTED_SIG))).unwrap();

		assert_eq!(pack.signer(), Some("1C3B7AE36BE79191"));
		assert_eq!(trusted_id(), "1C3B7AE36BE79191");
	}

	#[test]
	fn the_signature_should_not_change_the_digest() {
		assert_eq!(pack("x\n", Some(TRUSTED_SIG)).digest(), pack("x\n", None).digest());
	}

	#[test]
	fn verify_should_refuse_a_signature_for_other_files() {
		let error = trust(false, &[])
			.verify(pack("tampered\n", Some(TRUSTED_SIG)))
			.unwrap_err();

		assert!(error.to_string().contains("does not match"), "{error}");
	}

	#[test]
	fn verify_should_refuse_a_revoked_key_even_with_a_valid_signature() {
		let error = trust(false, &[&trusted_id().to_ascii_lowercase()])
			.verify(pack("x\n", Some(TRUSTED_SIG)))
			.unwrap_err();

		assert!(error.to_string().contains("revoked"), "{error}");
	}

	#[test]
	fn unsigned_and_untrusted_packs_should_load_unless_signatures_are_required() {
		let relaxed = trust(false, &[]);
		let strict = trust(true, &[]);

		assert_eq!(relaxed.verify(pack("x\n", None)).unwrap().signer(), None);
		assert_eq!(relaxed.verify(pack("x\n", Some(OTHER_SIG))).unwrap().signer(), None);
		assert!(strict.verify(pack("x\n", None)).is_err());
		assert!(
			strict
				.verify(pack("x\n", Some(OTHER_SIG)))
				.unwrap_err()
				.to_string()
				.contains("not trusted")
		);
	}

	#[test]
	fn verify_should_refuse_garbage_signatures() {
		assert!(trust(false, &[]).verify(pack("x\n", Some("not a signature"))).is_err());
	}

	#[test]
	fn trust_should_reject_invalid_keys() {
		let config = SigningConfig {
			keys: BTreeMap::from([("bad".to_owned(), "RWQnope".to_owned())]),
			..SigningConfig::default()
		};

		assert!(matches!(Trust::new(&config), Err(SourceError::Signature(_))));
	}

	#[test]
	fn base64_should_decode_with_and_without_padding() {
		assert_eq!(base64("aGk=").unwrap(), b"hi");
		assert_eq!(base64("aGV5").unwrap(), b"hey");
		assert_eq!(base64("aA=="), Some(b"h".to_vec()));
		assert_eq!(base64("a*=="), None);
	}
}
