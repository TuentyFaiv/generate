//! Login headers for private pack sources, built from the user config and environment variables.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use tfverse_core::workspace::HostAuth;

use super::fetch::Header;
use crate::SourceError;

type EnvLookup = dyn Fn(&str) -> Option<String> + Send + Sync;

/// Credentials per host; only names of environment variables are stored, never tokens.
#[derive(Clone)]
pub struct Credentials {
	hosts: BTreeMap<String, HostAuth>,
	env: Arc<EnvLookup>,
}

impl Default for Credentials {
	fn default() -> Self {
		Self::new(BTreeMap::new())
	}
}

impl fmt::Debug for Credentials {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Credentials")
			.field("hosts", &self.hosts.keys().collect::<Vec<_>>())
			.finish_non_exhaustive()
	}
}

impl Credentials {
	/// Credentials from the user config's `auth` table, reading tokens from the process environment.
	#[must_use]
	pub fn new(hosts: BTreeMap<String, HostAuth>) -> Self {
		Self {
			hosts: hosts
				.into_iter()
				.map(|(host, auth)| (host.to_ascii_lowercase(), auth))
				.collect(),
			env: Arc::new(|name| std::env::var(name).ok()),
		}
	}

	/// Reads tokens with `lookup` instead of the process environment.
	#[must_use]
	pub fn with_env(mut self, lookup: impl Fn(&str) -> Option<String> + Send + Sync + 'static) -> Self {
		self.env = Arc::new(lookup);
		self
	}

	/// Whether `host` has credentials configured.
	#[must_use]
	pub fn configured(&self, host: &str) -> bool {
		self.hosts.contains_key(&host.to_ascii_lowercase())
	}

	/// The headers to send to `host`; none when it has no credentials or its variable is unset.
	pub fn headers(&self, host: &str) -> Result<Vec<Header>, SourceError> {
		let Some(auth) = self.hosts.get(&host.to_ascii_lowercase()) else {
			return Ok(Vec::new());
		};
		let invalid = |reason: &str| SourceError::Auth(format!("`auth.\"{host}\"`: {reason}"));
		if !is_env_name(&auth.env) {
			return Err(invalid(
				"`env` must be an environment variable name such as `GITHUB_TOKEN`",
			));
		}
		if auth.header.is_empty() || !auth.header.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
			return Err(invalid("`header` must be a plain header name such as `Authorization`"));
		}
		if !auth.scheme.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
			return Err(invalid("`scheme` must be a plain word such as `Bearer`"));
		}
		let Some(token) = (self.env)(&auth.env).filter(|token| !token.is_empty()) else {
			return Ok(Vec::new());
		};
		if token.chars().any(|c| c.is_control() || c.is_whitespace()) {
			return Err(invalid(&format!("`{}` holds spaces or control characters", auth.env)));
		}
		let value = if auth.scheme.is_empty() {
			token
		} else {
			format!("{} {token}", auth.scheme)
		};
		Ok(vec![Header::new(auth.header.clone(), value)])
	}
}

fn is_env_name(name: &str) -> bool {
	(1..=128).contains(&name.len())
		&& !name.starts_with(|c: char| c.is_ascii_digit())
		&& name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
	use super::*;

	fn credentials(auth: HostAuth) -> Credentials {
		Credentials::new(BTreeMap::from([("GitHub.com".to_owned(), auth)]))
			.with_env(|name| (name == "GITHUB_TOKEN").then(|| "ghp_secret".to_owned()))
	}

	fn github_token() -> HostAuth {
		HostAuth {
			env: "GITHUB_TOKEN".to_owned(),
			header: "Authorization".to_owned(),
			scheme: "Bearer".to_owned(),
		}
	}

	#[test]
	fn headers_should_read_the_token_from_the_environment() {
		let headers = credentials(github_token()).headers("github.com").unwrap();

		assert_eq!(headers, [Header::new("Authorization", "Bearer ghp_secret")]);
	}

	#[test]
	fn headers_should_be_empty_for_other_hosts_and_unset_variables() {
		let unset = HostAuth {
			env: "UNSET_TOKEN".to_owned(),
			..github_token()
		};

		assert_eq!(credentials(github_token()).headers("example.com").unwrap(), []);
		assert_eq!(credentials(unset).headers("github.com").unwrap(), []);
	}

	#[test]
	fn headers_should_reject_header_injection() {
		let injected = HostAuth {
			header: "X-Token\r\nHost".to_owned(),
			..github_token()
		};

		assert!(matches!(
			credentials(injected).headers("github.com"),
			Err(SourceError::Auth(_))
		));
	}

	#[test]
	fn debug_should_not_show_tokens() {
		assert!(!format!("{:?}", credentials(github_token())).contains("ghp_secret"));
	}
}
