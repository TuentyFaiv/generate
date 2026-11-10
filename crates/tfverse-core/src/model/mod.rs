//! Base types every layer shares: paths, globs, digests, values and manifests.

mod case;
mod glob;
mod hash;
mod manifest;
mod path;
mod stack;
mod value;

pub(crate) use case::{Case, words};
pub use glob::{Glob, GlobError};
pub use hash::{Digest, ParseDigestError};
pub(crate) use manifest::SCHEMA_VERSION;
pub use manifest::{
	Dependency, Ecosystem, Fact, FileRule, Input, Migration, Name, NameError, PackManifest, PatchSpec, Permissions,
	PieceKind, PieceManifest, Position, Rename, Variant,
};
pub use path::{PathError, RelPath};
pub use stack::Stack;
pub use value::{Value, Values};
