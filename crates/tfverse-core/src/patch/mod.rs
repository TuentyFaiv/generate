//! Patches edit existing project files; each patch kind has a strategy that users can swap.

mod anchor;
mod command;
mod css;
mod import;

use std::collections::BTreeMap;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(crate) use anchor::AnchorPatcher;
pub(crate) use command::CommandPatcher;
pub(crate) use css::CssVarsPatcher;
pub(crate) use import::ImportPatcher;

use crate::model::{Position, RelPath};

/// A rendered edit to an existing file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Patch {
	/// Insert text next to an anchor line.
	Anchor {
		/// File to edit.
		file: RelPath,
		/// Text identifying the anchor line.
		anchor: String,
		/// Where to insert.
		position: Position,
		/// Text to insert.
		insert: String,
	},
	/// Add an import statement.
	Import {
		/// File to edit.
		file: RelPath,
		/// The import statement.
		import: String,
	},
	/// Add CSS custom properties to a block.
	CssVars {
		/// File to edit.
		file: RelPath,
		/// The block's selector or at-rule.
		selector: String,
		/// Properties without the leading `--`.
		vars: BTreeMap<String, String>,
	},
}

impl Patch {
	/// The file the patch edits.
	#[must_use]
	pub fn file(&self) -> &RelPath {
		match self {
			Self::Anchor { file, .. } | Self::Import { file, .. } | Self::CssVars { file, .. } => file,
		}
	}

	/// The patch kind.
	#[must_use]
	pub fn kind(&self) -> PatchKind {
		match self {
			Self::Anchor { .. } => PatchKind::Anchor,
			Self::Import { .. } => PatchKind::Import,
			Self::CssVars { .. } => PatchKind::CssVars,
		}
	}
}

/// Kinds of patch, each applied by one [`Patcher`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PatchKind {
	/// [`Patch::Anchor`].
	Anchor,
	/// [`Patch::Import`].
	Import,
	/// [`Patch::CssVars`].
	CssVars,
}

impl fmt::Display for PatchKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::Anchor => "anchor",
			Self::Import => "import",
			Self::CssVars => "css_vars",
		})
	}
}

/// The result of applying a patch to a file's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Patched {
	/// The file changed; `inserted` is the exact text now owned by the patch.
	Changed {
		/// New file text.
		text: String,
		/// Inserted region.
		inserted: String,
	},
	/// The file already contains the patch.
	AlreadyApplied {
		/// The region already present.
		inserted: String,
	},
}

/// Why a patch could not be applied.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PatchError {
	/// The anchor is not in the file.
	#[error("anchor `{0}` not found")]
	MissingAnchor(String),
	/// The anchor appears more than once.
	#[error("anchor `{anchor}` appears {count} times; it must be unique")]
	AmbiguousAnchor {
		/// The anchor text.
		anchor: String,
		/// How many lines contain it.
		count: usize,
	},
	/// The file or patch cannot be understood.
	#[error("{0}")]
	Invalid(String),
	/// An external patcher failed.
	#[error("external patcher `{name}` failed: {message}")]
	External {
		/// Patcher name.
		name: String,
		/// What went wrong.
		message: String,
	},
}

/// Applies one kind of patch.
pub trait Patcher: Send + Sync {
	/// Applies `patch` to `text`, the current content of `patch.file()`.
	fn apply(&self, text: &str, patch: &Patch) -> Result<Patched, PatchError>;
}

/// The patcher used for each patch kind.
pub(crate) struct Patchers {
	by_kind: BTreeMap<PatchKind, Box<dyn Patcher>>,
}

impl Patchers {
	pub(crate) fn builtin() -> Self {
		let mut by_kind: BTreeMap<PatchKind, Box<dyn Patcher>> = BTreeMap::new();
		by_kind.insert(PatchKind::Anchor, Box::new(AnchorPatcher));
		by_kind.insert(PatchKind::Import, Box::new(ImportPatcher));
		by_kind.insert(PatchKind::CssVars, Box::new(CssVarsPatcher));
		Self { by_kind }
	}

	pub(crate) fn set(&mut self, kind: PatchKind, patcher: Box<dyn Patcher>) {
		self.by_kind.insert(kind, patcher);
	}

	pub(crate) fn apply(&self, text: &str, patch: &Patch) -> Result<Patched, PatchError> {
		let kind = patch.kind();
		self
			.by_kind
			.get(&kind)
			.ok_or_else(|| PatchError::Invalid(format!("no patcher for `{kind}`")))?
			.apply(text, patch)
	}
}

impl fmt::Debug for Patchers {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.by_kind.keys()).finish()
	}
}

fn line_ending(text: &str) -> &'static str {
	if text.contains("\r\n") { "\r\n" } else { "\n" }
}
