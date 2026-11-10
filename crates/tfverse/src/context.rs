//! The project a command runs in: its engine, its user config and the packs it can use.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow, bail};
use tfverse_core::{Engine, Pack, UserConfig};
use tfverse_packs::{Library, PackSpec, Resolver};

/// An opened project.
pub struct Context {
	/// Project root.
	pub root: PathBuf,
	/// The user's config.
	pub user: UserConfig,
	/// The engine.
	pub engine: Engine,
	/// Official packs, the packs the lockfile pins, and any loaded by the command.
	pub library: Library,
}

impl Context {
	/// Opens the project at `root`, loading the official packs and, from the cache or their source,
	/// the pack versions its lockfile pins.
	pub fn open(root: &Path) -> anyhow::Result<Self> {
		let user = UserConfig::load()?;
		let engine = Engine::builder()
			.user_config(user.clone())
			.open(root)
			.with_context(|| format!("cannot open the project at {}", root.display()))?;
		let mut context = Self {
			root: root.to_path_buf(),
			user,
			engine,
			library: Library::official()?,
		};
		let (pinned, _unavailable) = context.resolver()?.pinned(&context.engine.lockfile()?);
		for pack in pinned {
			context.library.add(pack);
		}
		Ok(context)
	}

	/// A resolver with the project's namespaces and the user's credentials and trusted keys.
	pub fn resolver(&self) -> anyhow::Result<Resolver> {
		Ok(Resolver::for_project(&self.root, self.engine.config(), &self.user)?)
	}

	/// Loads `source`; `pinned` uses the version the lockfile recorded for it.
	pub fn load(&mut self, source: &str, pinned: bool) -> anyhow::Result<Pack> {
		let canonical = source.parse::<PackSpec>()?.to_string();
		let digest = if pinned {
			self
				.engine
				.lockfile()?
				.packs
				.iter()
				.find(|record| record.source.as_deref() == Some(canonical.as_str()))
				.map(|record| record.digest)
		} else {
			None
		};
		let pack = self.resolver()?.resolve(source, digest)?;
		self.library.add(pack.clone());
		Ok(pack)
	}

	/// The pack and piece named by `name`, `pack/piece` or a bare piece name found in exactly one pack.
	pub fn piece<'a>(&'a self, name: &'a str, from: Option<&'a Pack>) -> anyhow::Result<(&'a Pack, &'a str)> {
		if let Some(pack) = from {
			let piece = name.rsplit('/').next().unwrap_or(name);
			return pack
				.piece(piece)
				.map(|_| (pack, piece))
				.ok_or_else(|| anyhow!("pack `{}` has no piece `{piece}`", pack.manifest().name));
		}
		if let Some((pack, piece)) = name.split_once('/') {
			let found = self
				.library
				.get(pack)
				.ok_or_else(|| anyhow!("no pack `{pack}` is loaded; use --from <source> to load it"))?;
			return found
				.piece(piece)
				.map(|_| (found, piece))
				.ok_or_else(|| anyhow!("pack `{pack}` has no piece `{piece}`"));
		}
		let matches: Vec<&Pack> = self.library.iter().filter(|pack| pack.piece(name).is_some()).collect();
		match matches.as_slice() {
			[pack] => Ok((pack, name)),
			[] => bail!("no loaded pack has a piece `{name}`; see `tfverse pack list`"),
			many => bail!(
				"several packs have a piece `{name}` ({}); write it as `pack/{name}`",
				many
					.iter()
					.map(|pack| pack.manifest().name.as_str())
					.collect::<Vec<_>>()
					.join(", ")
			),
		}
	}
}
