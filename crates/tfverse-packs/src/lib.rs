//! Pack sources for `tfverse`: official packs built into the binary, folders, GitHub tags, HTTPS
//! tarballs and git repositories, each verified against the hash the lockfile pinned and, when
//! signed, against the minisign keys the user trusts.
//!
//! Network sources need the default `remote` feature; without it the HTTPS crates are left out,
//! `git:` is refused, and remote sources load only from the cache.
//!
//! Layers, each using only those above it: `error` and `spec` (source strings), `transport`
//! (downloads, git, credentials), `format` (archives, registry items), `signing`, `store` (cache,
//! official packs, library) and `resolver`.

mod error;
mod format;
mod resolver;
mod signing;
mod spec;
mod store;
pub mod transport;

pub use error::SourceError;
pub use resolver::Resolver;
pub use signing::Trust;
pub use spec::{PackSpec, SpecError};
pub use store::{Cache, Library};
pub use transport::Credentials;
