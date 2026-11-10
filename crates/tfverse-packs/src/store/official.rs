//! The official packs, built into the binary from the repository's `packs/` folder.

use std::collections::BTreeMap;

use tfverse_core::Pack;
use tfverse_core::pack::PackError;

use crate::spec::PackSpec;

/// A pack's files: path and contents.
type Files = &'static [(&'static str, &'static [u8])];

include!(concat!(env!("OUT_DIR"), "/official.rs"));

/// Names of the official packs.
pub fn names() -> impl Iterator<Item = &'static str> {
	OFFICIAL.iter().map(|(name, _)| *name)
}

/// Loads the official pack `name`, if there is one.
#[must_use]
pub fn load(name: &str) -> Option<Result<Pack, PackError>> {
	let (_, files) = OFFICIAL.iter().find(|(official, _)| *official == name)?;
	let files: BTreeMap<String, Vec<u8>> = files
		.iter()
		.map(|(path, bytes)| ((*path).to_owned(), bytes.to_vec()))
		.collect();
	Some(Pack::from_files(files).map(|pack| pack.with_source(PackSpec::Official(name.to_owned()).to_string())))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn every_official_pack_should_load() {
		let names: Vec<&str> = names().collect();
		assert!(names.contains(&"ui") && names.contains(&"backend"), "{names:?}");

		for name in names {
			let pack = load(name).unwrap().unwrap();

			assert_eq!(pack.manifest().name.as_str(), name);
			assert_eq!(pack.source(), Some(format!("official:{name}").as_str()));
		}
	}
}
