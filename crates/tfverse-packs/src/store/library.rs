//! The packs available in a session.

use tfverse_core::Pack;
use tfverse_core::engine::PackProvider;
use tfverse_core::model::Name;
use tfverse_core::pack::PackError;

use super::official;
use semver::Version;

/// The official packs plus any loaded later, keeping every version so updates can reach the old one.
#[derive(Clone, Debug, Default)]
pub struct Library {
	packs: Vec<Pack>,
}

impl Library {
	/// Every official pack.
	pub fn official() -> Result<Self, PackError> {
		let mut library = Self::default();
		for name in official::names() {
			if let Some(pack) = official::load(name) {
				library.add(pack?);
			}
		}
		Ok(library)
	}

	/// Adds `pack`, replacing the same version of it.
	pub fn add(&mut self, pack: Pack) {
		let key = |pack: &Pack| (pack.manifest().name.clone(), pack.manifest().version.clone());
		let added = key(&pack);
		self.packs.retain(|loaded| key(loaded) != added);
		self.packs.push(pack);
		self.packs.sort_by_key(key);
	}

	/// The newest version of the pack called `name`.
	#[must_use]
	pub fn get(&self, name: &str) -> Option<&Pack> {
		self
			.packs
			.iter()
			.rev()
			.find(|pack| pack.manifest().name.as_str() == name)
	}

	/// Every loaded version of `name`, oldest first.
	pub fn versions<'a>(&'a self, name: &'a str) -> impl DoubleEndedIterator<Item = &'a Pack> {
		self
			.packs
			.iter()
			.filter(move |pack| pack.manifest().name.as_str() == name)
	}

	/// The newest version of every pack, by name.
	pub fn iter(&self) -> impl Iterator<Item = &Pack> {
		let mut newest: Vec<&Pack> = Vec::new();
		for pack in &self.packs {
			match newest.last_mut() {
				Some(last) if last.manifest().name == pack.manifest().name => *last = pack,
				_ => newest.push(pack),
			}
		}
		newest.into_iter()
	}
}

impl PackProvider for Library {
	fn pack(&self, name: &Name, version: &Version) -> Option<&Pack> {
		self.packs.as_slice().pack(name, version)
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use super::*;

	fn pack(version: &str) -> Pack {
		let manifest = format!("schema_version = 1\nname = \"ui\"\nversion = \"{version}\"\n");
		Pack::from_files(BTreeMap::from([("pack.toml".to_owned(), manifest.into_bytes())])).unwrap()
	}

	#[test]
	fn library_should_keep_every_version_and_prefer_the_newest() {
		let mut library = Library::default();
		library.add(pack("1.1.0"));
		library.add(pack("1.0.0"));

		let old = Version::new(1, 0, 0);
		assert_eq!(library.get("ui").unwrap().manifest().version.to_string(), "1.1.0");
		assert!(
			library
				.pack(&library.get("ui").unwrap().manifest().name.clone(), &old)
				.is_some()
		);
		assert_eq!(library.iter().count(), 1);
	}
}
