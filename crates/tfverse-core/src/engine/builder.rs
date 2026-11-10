//! [`EngineBuilder`]: where an engine gets its files, user config, patchers and event sink.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use super::{Engine, EngineError};
use crate::event::{EventSink, NoopSink};
use crate::migrate::{AstGrep, AstGrepCommand};
use crate::patch::{CommandPatcher, PatchKind, Patcher, Patchers};
use crate::workspace::{Fs, ProjectConfig, UserConfig, Workspace};

/// Configures and opens an [`Engine`].
#[must_use]
pub struct EngineBuilder {
	user: Option<UserConfig>,
	sink: Arc<dyn EventSink>,
	patchers: Vec<(PatchKind, Box<dyn Patcher>)>,
	ast_grep: Option<Arc<dyn AstGrep>>,
}

impl Default for EngineBuilder {
	fn default() -> Self {
		Self {
			user: None,
			sink: Arc::new(NoopSink),
			patchers: Vec::new(),
			ast_grep: None,
		}
	}
}

impl fmt::Debug for EngineBuilder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("EngineBuilder")
			.field("user", &self.user)
			.field(
				"patchers",
				&self.patchers.iter().map(|(kind, _)| kind).collect::<Vec<_>>(),
			)
			.finish_non_exhaustive()
	}
}

impl EngineBuilder {
	/// Uses `user` instead of loading the user's config file.
	pub fn user_config(mut self, user: UserConfig) -> Self {
		self.user = Some(user);
		self
	}

	/// Sends engine events to `sink`.
	pub fn sink(mut self, sink: impl EventSink + 'static) -> Self {
		self.sink = Arc::new(sink);
		self
	}

	/// Uses `patcher` for `kind`, over both the built-in and the user's configured one.
	pub fn patcher(mut self, kind: PatchKind, patcher: impl Patcher + 'static) -> Self {
		self.patchers.push((kind, Box::new(patcher)));
		self
	}

	/// Uses `ast_grep` for ast-grep migrations instead of the configured program.
	pub fn ast_grep(mut self, ast_grep: impl AstGrep + 'static) -> Self {
		self.ast_grep = Some(Arc::new(ast_grep));
		self
	}

	/// Opens the project folder at `root`.
	pub fn open(self, root: &Path) -> Result<Engine, EngineError> {
		self.build(Workspace::open(root)?)
	}

	/// Opens a project on any [`Fs`], such as a [`MemoryFs`](crate::workspace::MemoryFs) preview.
	pub fn open_fs(self, fs: Arc<dyn Fs>) -> Result<Engine, EngineError> {
		self.build(Workspace::new(fs))
	}

	fn build(self, workspace: Workspace) -> Result<Engine, EngineError> {
		let user = match self.user {
			Some(user) => user,
			None => UserConfig::load()?,
		};
		let mut patchers = Patchers::builtin();
		for (kind, external) in &user.patchers {
			let patcher = CommandPatcher::new(&kind.to_string(), &external.command)
				.map_err(|source| EngineError::Patcher { kind: *kind, source })?;
			patchers.set(*kind, Box::new(patcher));
		}
		for (kind, patcher) in self.patchers {
			patchers.set(kind, patcher);
		}
		let config = ProjectConfig::load(&workspace)?;
		let ast_grep = self.ast_grep.unwrap_or_else(|| {
			Arc::new(
				user
					.ast_grep
					.as_ref()
					.map_or_else(AstGrepCommand::default, |tool| AstGrepCommand::new(&tool.command)),
			)
		});
		Ok(Engine {
			workspace,
			config,
			patchers,
			ast_grep,
			sink: self.sink,
		})
	}
}
