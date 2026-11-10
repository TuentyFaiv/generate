//! Reads `.tar.gz` and `.tar` archives in memory, keeping only regular files.

use std::collections::BTreeMap;
use std::io::Read;

use flate2::read::GzDecoder;
use tar::{Archive, EntryType};

use crate::transport::MAX_DOWNLOAD;

const MAX_ENTRIES: usize = 20_000;

/// Why an archive could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
	/// Not a valid gzip-compressed tar archive.
	#[error("invalid archive: {0}")]
	Invalid(String),
	/// Too many entries or too much content.
	#[error("archive exceeds a limit: {0}")]
	Limit(String),
}

/// Every regular file in `bytes`, keyed by `/`-separated path.
///
/// With `strip_root`, the single top folder that GitHub and most release archives use is removed.
/// Links and special files are skipped, never followed.
pub fn unpack(bytes: &[u8], strip_root: bool) -> Result<BTreeMap<String, Vec<u8>>, ArchiveError> {
	read(Archive::new(GzDecoder::new(bytes)), strip_root)
}

/// Like [`unpack`], for an uncompressed tar such as `git archive` writes.
pub fn unpack_tar(bytes: &[u8], strip_root: bool) -> Result<BTreeMap<String, Vec<u8>>, ArchiveError> {
	read(Archive::new(bytes), strip_root)
}

fn read<R: Read>(mut archive: Archive<R>, strip_root: bool) -> Result<BTreeMap<String, Vec<u8>>, ArchiveError> {
	let invalid = |error: std::io::Error| ArchiveError::Invalid(error.to_string());
	let mut files = BTreeMap::new();
	let mut total: u64 = 0;
	for (count, entry) in archive.entries().map_err(invalid)?.enumerate() {
		if count >= MAX_ENTRIES {
			return Err(ArchiveError::Limit(format!("more than {MAX_ENTRIES} entries")));
		}
		let mut entry = entry.map_err(invalid)?;
		if entry.header().entry_type() != EntryType::Regular {
			continue;
		}
		let path = entry.path().map_err(invalid)?.to_string_lossy().replace('\\', "/");
		let path = if strip_root {
			match path.split_once('/') {
				Some((_, rest)) => rest.to_owned(),
				None => continue,
			}
		} else {
			path
		};
		total += entry.size();
		if total > MAX_DOWNLOAD {
			return Err(ArchiveError::Limit(format!("more than {MAX_DOWNLOAD} bytes unpacked")));
		}
		let mut contents = Vec::new();
		entry.read_to_end(&mut contents).map_err(invalid)?;
		files.insert(path, contents);
	}
	Ok(files)
}

/// The files under `dir/`, with that prefix removed.
#[must_use]
pub fn subtree(files: BTreeMap<String, Vec<u8>>, dir: &str) -> BTreeMap<String, Vec<u8>> {
	let prefix = format!("{}/", dir.trim_end_matches('/'));
	files
		.into_iter()
		.filter_map(|(path, bytes)| path.strip_prefix(&prefix).map(|rest| (rest.to_owned(), bytes)))
		.collect()
}

#[cfg(test)]
pub(crate) mod tests {
	use flate2::Compression;
	use flate2::write::GzEncoder;

	use super::*;

	/// A `.tar.gz` with `files`, plus a symlink that must be ignored.
	pub(crate) fn tarball(files: &[(&str, &str)]) -> Vec<u8> {
		let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
		for (path, text) in files {
			let mut header = tar::Header::new_gnu();
			header.set_size(text.len() as u64);
			header.set_mode(0o644);
			header.set_cksum();
			builder.append_data(&mut header, path, text.as_bytes()).unwrap();
		}
		let mut link = tar::Header::new_gnu();
		link.set_entry_type(EntryType::Symlink);
		link.set_size(0);
		link.set_cksum();
		builder.append_link(&mut link, "root/evil", "/etc/passwd").unwrap();
		builder.into_inner().unwrap().finish().unwrap()
	}

	#[test]
	fn unpack_should_strip_the_root_and_skip_links() {
		let files = unpack(&tarball(&[("root/pack.toml", "x"), ("root/a/b.txt", "y")]), true).unwrap();

		assert_eq!(files.keys().collect::<Vec<_>>(), ["a/b.txt", "pack.toml"]);
	}

	#[test]
	fn subtree_should_keep_one_folder() {
		let files = unpack(
			&tarball(&[("root/packs/ui/pack.toml", "x"), ("root/other.txt", "y")]),
			true,
		)
		.unwrap();

		assert_eq!(subtree(files, "packs/ui").keys().collect::<Vec<_>>(), ["pack.toml"]);
	}

	#[test]
	fn unpack_should_reject_garbage() {
		assert!(matches!(unpack(b"not an archive", true), Err(ArchiveError::Invalid(_))));
	}
}
