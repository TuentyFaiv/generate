//! Downloading pack archives: over HTTPS with the `remote` feature, verified against the operating
//! system's trusted certificates.

use std::fmt;

#[cfg(feature = "remote")]
pub use https::HttpsFetch;

/// The largest archive tfverse downloads.
pub const MAX_DOWNLOAD: u64 = 64 * 1024 * 1024;

/// The most redirects one download follows.
#[cfg(feature = "remote")]
const MAX_REDIRECTS: usize = 5;

/// Downloads a URL; a trait so tests and offline tools can serve archives themselves.
pub trait Fetch: Send + Sync + fmt::Debug {
	/// The body of `url`, sending `headers` to its host only, never to a host it redirects to.
	fn get(&self, url: &str, headers: &[Header]) -> Result<Vec<u8>, FetchError>;
}

/// A request header whose value is never printed.
#[derive(Clone, PartialEq, Eq)]
pub struct Header {
	name: String,
	value: String,
}

impl Header {
	/// A header called `name` carrying `value`.
	#[must_use]
	pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
		Self {
			name: name.into(),
			value: value.into(),
		}
	}

	/// The header name.
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	/// The header value; keep it out of logs and errors.
	#[must_use]
	pub fn value(&self) -> &str {
		&self.value
	}
}

impl fmt::Debug for Header {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: ***", self.name)
	}
}

/// The host of an `https://` URL, lowercased and without a port.
#[must_use]
pub fn host(url: &str) -> Option<String> {
	let rest = url.strip_prefix("https://")?;
	let authority = rest.split(['/', '?', '#']).next()?;
	let host = authority.rsplit('@').next()?.split(':').next()?;
	(!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// Where a redirect from `from` to `location` leads; only HTTPS targets are allowed.
#[cfg(any(feature = "remote", test))]
fn redirect(from: &str, location: &str) -> Option<String> {
	if location.starts_with("https://") {
		return Some(location.to_owned());
	}
	if location.starts_with('/') && !location.starts_with("//") {
		let rest = from.strip_prefix("https://")?;
		let authority = rest.split(['/', '?', '#']).next()?;
		return Some(format!("https://{authority}{location}"));
	}
	None
}

/// Why a download failed.
#[derive(Debug, thiserror::Error)]
#[error("could not download `{url}`: {reason}")]
pub struct FetchError {
	/// The URL.
	pub url: String,
	/// What went wrong.
	pub reason: String,
}

/// Refuses every download, for offline use and builds without the `remote` feature.
#[derive(Clone, Copy, Debug, Default)]
pub struct Offline;

impl Fetch for Offline {
	fn get(&self, url: &str, _headers: &[Header]) -> Result<Vec<u8>, FetchError> {
		Err(FetchError {
			url: url.to_owned(),
			reason: if cfg!(feature = "remote") {
				"downloads are turned off".to_owned()
			} else {
				"this build of tfverse cannot download packs".to_owned()
			},
		})
	}
}

#[cfg(feature = "remote")]
mod https {
	use std::fmt;
	use std::io::Read;
	use std::sync::{Arc, OnceLock};
	use std::time::Duration;

	use ureq::Agent;
	use ureq::tls::{Certificate, RootCerts, TlsConfig, TlsProvider};

	use super::{Fetch, FetchError, Header, MAX_DOWNLOAD, MAX_REDIRECTS, host, redirect};

	const TIMEOUT: Duration = Duration::from_secs(60);

	/// HTTPS downloads with rustls, trusting the certificates the operating system trusts.
	#[derive(Default)]
	pub struct HttpsFetch {
		agent: OnceLock<Agent>,
	}

	impl fmt::Debug for HttpsFetch {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("HttpsFetch").finish_non_exhaustive()
		}
	}

	impl HttpsFetch {
		/// The agent, created on the first download because reading the OS certificates takes a moment.
		fn agent(&self) -> &Agent {
			self.agent.get_or_init(|| {
				let roots: Vec<Certificate<'static>> = rustls_native_certs::load_native_certs()
					.certs
					.iter()
					.map(|der| Certificate::from_der(der.as_ref()).to_owned())
					.collect();
				let tls = TlsConfig::builder()
					.provider(TlsProvider::Rustls)
					.root_certs(RootCerts::from(roots))
					.unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
					.build();
				Agent::config_builder()
					.https_only(true)
					.max_redirects(0)
					.http_status_as_error(false)
					.timeout_global(Some(TIMEOUT))
					.user_agent(concat!("tfverse/", env!("CARGO_PKG_VERSION")))
					.tls_config(tls)
					.build()
					.into()
			})
		}
	}

	impl Fetch for HttpsFetch {
		fn get(&self, url: &str, headers: &[Header]) -> Result<Vec<u8>, FetchError> {
			let error = |reason: String| FetchError {
				url: url.to_owned(),
				reason,
			};
			if !url.starts_with("https://") {
				return Err(error("only HTTPS is allowed".to_owned()));
			}
			let origin = host(url);
			let mut current = url.to_owned();
			let mut response = None;
			for _ in 0..=MAX_REDIRECTS {
				let mut request = self.agent().get(&current);
				if host(&current) == origin {
					for header in headers {
						request = request.header(header.name(), header.value());
					}
				}
				let reply = request.call().map_err(|source| error(source.to_string()))?;
				let status = reply.status().as_u16();
				if !(300..400).contains(&status) {
					if status >= 400 {
						return Err(error(format!("the server answered HTTP {status}")));
					}
					response = Some(reply);
					break;
				}
				let location = reply
					.headers()
					.get("location")
					.and_then(|value| value.to_str().ok())
					.ok_or_else(|| error(format!("HTTP {status} without a location")))?;
				current =
					redirect(&current, location).ok_or_else(|| error("redirected to a location that is not HTTPS".to_owned()))?;
			}
			let response = response.ok_or_else(|| error(format!("more than {MAX_REDIRECTS} redirects")))?;
			let mut body = Vec::new();
			response
				.into_body()
				.into_reader()
				.take(MAX_DOWNLOAD + 1)
				.read_to_end(&mut body)
				.map_err(|source| error(source.to_string()))?;
			if body.len() as u64 > MAX_DOWNLOAD {
				return Err(error(format!("larger than {MAX_DOWNLOAD} bytes")));
			}
			Ok(body)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn host_should_ignore_ports_users_and_paths() {
		assert_eq!(
			host("https://User@API.GitHub.com:443/repos?x").as_deref(),
			Some("api.github.com")
		);
		assert_eq!(host("http://example.com/"), None);
	}

	#[test]
	fn redirect_should_follow_only_https() {
		assert_eq!(
			redirect("https://api.github.com/repos/a/b", "/x").as_deref(),
			Some("https://api.github.com/x")
		);
		assert_eq!(
			redirect("https://a.com/", "https://codeload.github.com/z").as_deref(),
			Some("https://codeload.github.com/z")
		);
		assert_eq!(redirect("https://a.com/", "http://a.com/"), None);
		assert_eq!(redirect("https://a.com/", "//evil.com/"), None);
	}

	#[test]
	fn header_debug_should_hide_the_value() {
		assert_eq!(
			format!("{:?}", Header::new("Authorization", "Bearer secret")),
			"Authorization: ***"
		);
	}
}
