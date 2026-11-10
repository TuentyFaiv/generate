//! Network access, behind interfaces so tests and embedders can supply their own: HTTPS downloads,
//! the user's git over SSH, and the credentials sent with them.

mod auth;
mod fetch;
mod git;

pub use auth::Credentials;
#[cfg(feature = "remote")]
pub use fetch::HttpsFetch;
pub use fetch::{Fetch, FetchError, Header, MAX_DOWNLOAD, Offline, host};
pub use git::{GIT_TIMEOUT, Git, NoGit, SystemGit};

#[cfg(test)]
pub(crate) use git::tests as git_tests;
