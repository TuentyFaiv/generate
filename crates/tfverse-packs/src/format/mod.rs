//! Turning downloaded bytes into pack files: `.tar.gz` and `.tar` archives, and registry items.

mod archive;
mod registry;

pub use archive::ArchiveError;
pub(crate) use archive::{subtree, unpack, unpack_tar};
pub(crate) use registry::{Project, pack_files};

#[cfg(test)]
pub(crate) use archive::tests as archive_tests;
