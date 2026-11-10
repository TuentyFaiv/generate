//! Where a pack comes from, written as one string such as `gh:acme/packs@v1.2.0`.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

const OFFICIAL: &str = "official:";
const GITHUB: &str = "gh:";
const GIT: &str = "git:";
const SSH: &str = "ssh://";
const REGISTRY: &str = "registry:";
const PATH: &str = "path:";

/// A pack source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackSpec {
	/// A pack built into tfverse, e.g. `ui` or `official:ui`.
	Official(String),
	/// A folder on this machine, e.g. `./packs/mine` or `path:/opt/packs/mine`.
	Path(PathBuf),
	/// A GitHub repository at a tag or commit, e.g. `gh:acme/packs@v1.2.0`, optionally a folder inside it.
	GitHub {
		/// Owner or organization.
		owner: String,
		/// Repository.
		repo: String,
		/// Folder inside the repository, if the pack is not at its root.
		dir: Option<String>,
		/// Tag or commit.
		reference: String,
	},
	/// A `.tar.gz` archive served over HTTPS.
	Tarball(String),
	/// Any git repository over SSH at a tag or commit, fetched with the user's git, e.g.
	/// `git:git@github.com:acme/packs.git@v1.2.0`; `//folder` after the URL picks a folder inside it.
	Git {
		/// `ssh://` or `user@host:path` URL.
		url: String,
		/// Folder inside the repository, if the pack is not at its root.
		dir: Option<String>,
		/// Tag or commit.
		reference: String,
	},
	/// An item in the `registry-item.json` format: its HTTPS URL, or `@namespace/name` from the
	/// project's `registry.namespaces`, e.g. `registry:@acme/card`.
	Registry(String),
	/// A pack in a namespace configured in the project, e.g. `@acme/ui`.
	Alias {
		/// Namespace with its `@`.
		namespace: String,
		/// Pack name inside the namespace.
		pack: String,
	},
}

/// Why a source string was rejected.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("invalid pack source `{text}`: {reason}")]
pub struct SpecError {
	text: String,
	reason: &'static str,
}

impl FromStr for PackSpec {
	type Err = SpecError;

	fn from_str(text: &str) -> Result<Self, Self::Err> {
		let error = |reason| SpecError {
			text: text.to_owned(),
			reason,
		};
		if text.is_empty() || text.len() > 512 || text.chars().any(char::is_control) {
			return Err(error("must be 1 to 512 printable characters"));
		}
		if let Some(name) = text.strip_prefix(OFFICIAL) {
			return official(name).ok_or_else(|| error("official packs have plain lowercase names"));
		}
		if let Some(path) = text.strip_prefix(PATH) {
			return Ok(Self::Path(PathBuf::from(path)));
		}
		if text.starts_with("./") || text.starts_with("../") || text.starts_with('/') || text.starts_with(".\\") {
			return Ok(Self::Path(PathBuf::from(text)));
		}
		if let Some(rest) = text.strip_prefix(GITHUB) {
			return github(rest).ok_or_else(|| error("expected `gh:owner/repo[/folder]@tag`"));
		}
		if let Some(item) = text.strip_prefix(REGISTRY) {
			return registry(item).ok_or_else(|| error("expected `registry:https://….json` or `registry:@namespace/name`"));
		}
		if let Some(rest) = text.strip_prefix(GIT) {
			if rest.starts_with("https://") || rest.starts_with("http://") {
				return Err(error("`git:` sources use SSH; for HTTPS use `gh:` or a tarball URL"));
			}
			return git(rest).ok_or_else(|| error("expected `git:<ssh:// or user@host:path URL>[//folder]@tag`"));
		}
		if text.starts_with("https://") {
			let path = text.split(['?', '#']).next().unwrap_or(text).to_ascii_lowercase();
			return if [".tar.gz", ".tgz"].iter().any(|suffix| path.ends_with(suffix)) {
				Ok(Self::Tarball(text.to_owned()))
			} else {
				Err(error("tarball URLs must end in `.tar.gz` or `.tgz`"))
			};
		}
		if text.starts_with("http://") {
			return Err(error("packs are only downloaded over HTTPS"));
		}
		if let Some(rest) = text.strip_prefix('@') {
			let (namespace, pack) = rest
				.split_once('/')
				.ok_or_else(|| error("expected `@namespace/pack`"))?;
			return if is_name(namespace) && is_name(pack) {
				Ok(Self::Alias {
					namespace: format!("@{namespace}"),
					pack: pack.to_owned(),
				})
			} else {
				Err(error("namespaces and packs have plain lowercase names"))
			};
		}
		official(text).ok_or_else(|| error("not an official pack name, path, `gh:`, `https://` or `@namespace/` source"))
	}
}

impl fmt::Display for PackSpec {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Official(name) => write!(f, "{OFFICIAL}{name}"),
			Self::Path(path) => write!(f, "{PATH}{}", path.display()),
			Self::GitHub {
				owner,
				repo,
				dir,
				reference,
			} => {
				write!(f, "{GITHUB}{owner}/{repo}")?;
				if let Some(dir) = dir {
					write!(f, "/{dir}")?;
				}
				write!(f, "@{reference}")
			}
			Self::Tarball(url) => f.write_str(url),
			Self::Registry(item) => write!(f, "{REGISTRY}{item}"),
			Self::Git { url, dir, reference } => {
				write!(f, "{GIT}{url}")?;
				if let Some(dir) = dir {
					write!(f, "//{dir}")?;
				}
				write!(f, "@{reference}")
			}
			Self::Alias { namespace, pack } => write!(f, "{namespace}/{pack}"),
		}
	}
}

impl PackSpec {
	/// The HTTPS archive to download, for remote sources.
	#[must_use]
	pub fn archive_url(&self) -> Option<String> {
		match self {
			Self::GitHub {
				owner, repo, reference, ..
			} => Some(format!("https://codeload.github.com/{owner}/{repo}/tar.gz/{reference}")),
			Self::Tarball(url) => Some(url.clone()),
			Self::Official(_) | Self::Path(_) | Self::Git { .. } | Self::Registry(_) | Self::Alias { .. } => None,
		}
	}
}

/// The API archive of a GitHub source, which accepts a token; the plain archive does not.
#[must_use]
pub(crate) fn github_api_url(spec: &PackSpec) -> Option<String> {
	match spec {
		PackSpec::GitHub {
			owner, repo, reference, ..
		} => Some(format!(
			"https://api.github.com/repos/{owner}/{repo}/tarball/{reference}"
		)),
		_ => None,
	}
}

fn official(name: &str) -> Option<PackSpec> {
	is_name(name).then(|| PackSpec::Official(name.to_owned()))
}

fn github(rest: &str) -> Option<PackSpec> {
	let (path, reference) = rest.rsplit_once('@')?;
	let mut parts = path.splitn(3, '/');
	let owner = parts.next()?;
	let repo = parts.next()?;
	let dir = parts.next().map(|dir| dir.trim_end_matches('/').to_owned());
	let valid_part = |part: &str| {
		!part.is_empty()
			&& part.len() <= 100
			&& !part.starts_with('.')
			&& part
				.chars()
				.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
	};
	let valid_dir = dir.as_deref().is_none_or(|dir| dir.split('/').all(valid_part));
	(valid_part(owner) && valid_part(repo) && valid_dir && is_reference(reference)).then(|| PackSpec::GitHub {
		owner: owner.to_owned(),
		repo: repo.to_owned(),
		dir,
		reference: reference.to_owned(),
	})
}

fn git(rest: &str) -> Option<PackSpec> {
	let (location, reference) = rest.rsplit_once('@')?;
	let scheme = if location.starts_with(SSH) { SSH.len() } else { 0 };
	if scheme == 0 && location.contains("://") {
		return None;
	}
	let (url, dir) = match location[scheme..].find("//") {
		Some(at) => (&location[..scheme + at], Some(&location[scheme + at + 2..])),
		None => (location, None),
	};
	let scp_like = scheme == 0 && url.split_once(':').is_some_and(|(host, _)| !host.contains('/'));
	let valid_url = (scheme > 0 || scp_like)
		&& !url.starts_with('-')
		&& !url.contains("::")
		&& url.len() > scheme
		&& url
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | ':' | '/' | '@' | '%' | '+'));
	let valid_dir = dir.is_none_or(|dir| {
		!dir.is_empty()
			&& dir.split('/').all(|part| {
				!part.is_empty()
					&& !part.starts_with('.')
					&& part
						.chars()
						.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
			})
	});
	(valid_url && valid_dir && is_reference(reference)).then(|| PackSpec::Git {
		url: url.to_owned(),
		dir: dir.map(str::to_owned),
		reference: reference.to_owned(),
	})
}

fn registry(item: &str) -> Option<PackSpec> {
	let valid = if let Some(url) = item.strip_prefix("https://") {
		let path = url.split(['?', '#']).next().unwrap_or(url);
		path.to_ascii_lowercase().ends_with(".json") && !item.chars().any(char::is_whitespace)
	} else {
		item
			.strip_prefix('@')
			.and_then(|rest| rest.split_once('/'))
			.is_some_and(|(namespace, name)| is_name(namespace) && is_name(name))
	};
	valid.then(|| PackSpec::Registry(item.to_owned()))
}

fn is_reference(reference: &str) -> bool {
	!reference.is_empty()
		&& reference.len() <= 128
		&& !reference.starts_with(['-', '.'])
		&& !reference.contains("..")
		&& reference
			.chars()
			.all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
}

fn is_name(text: &str) -> bool {
	(1..=64).contains(&text.len())
		&& text.starts_with(|c: char| c.is_ascii_lowercase())
		&& text
			.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
	use super::*;

	fn spec(text: &str) -> PackSpec {
		text.parse().unwrap()
	}

	#[test]
	fn parse_should_read_every_kind_of_source() {
		assert_eq!(spec("ui"), PackSpec::Official("ui".to_owned()));
		assert_eq!(spec("./packs/mine"), PackSpec::Path(PathBuf::from("./packs/mine")));
		assert_eq!(
			spec("gh:acme/packs/ui@v1.2.0"),
			PackSpec::GitHub {
				owner: "acme".to_owned(),
				repo: "packs".to_owned(),
				dir: Some("ui".to_owned()),
				reference: "v1.2.0".to_owned(),
			}
		);
		assert_eq!(
			spec("@acme/ui"),
			PackSpec::Alias {
				namespace: "@acme".to_owned(),
				pack: "ui".to_owned(),
			}
		);
		assert!(matches!(
			spec("https://example.com/ui-1.0.0.tar.gz"),
			PackSpec::Tarball(_)
		));
	}

	#[test]
	fn display_should_round_trip() {
		for text in [
			"official:ui",
			"path:./packs/mine",
			"gh:acme/packs/ui@v1.2.0",
			"@acme/ui",
			"git:git@github.com:acme/packs.git@v1.2.0",
			"git:git@gitlab.com:acme/packs.git//packs/ui@v2",
			"registry:@acme/card",
			"registry:https://acme.dev/r/card.json",
		] {
			assert_eq!(spec(text).to_string(), text);
		}
	}

	#[test]
	fn parse_should_reject_unpinned_and_insecure_sources() {
		for text in [
			"gh:acme/packs",
			"http://example.com/x.tar.gz",
			"https://example.com/pack",
			"gh:acme/packs@--upload-pack",
			"git:-oProxyCommand=x:y@v1",
			"git:ext::sh -c touch@v1",
			"git:file:///tmp/repo@v1",
			"git:/tmp/repo@v1",
			"git:http://example.com/repo.git@v1",
			"git:https://gitlab.com/acme/packs.git@v1",
			"registry:http://acme.dev/r/card.json",
			"registry:https://acme.dev/r/card",
			"registry:../card",
			"registry:card",
			"git:git@github.com:acme/packs.git",
		] {
			assert!(text.parse::<PackSpec>().is_err(), "{text}");
		}
	}

	#[test]
	fn github_should_download_the_tag_archive() {
		assert_eq!(
			spec("gh:acme/packs@v1.2.0").archive_url().unwrap(),
			"https://codeload.github.com/acme/packs/tar.gz/v1.2.0"
		);
	}

	#[test]
	fn git_should_split_the_url_folder_and_reference() {
		assert_eq!(
			spec("git:ssh://git@example.com:2222/acme/packs.git//ui@abc123"),
			PackSpec::Git {
				url: "ssh://git@example.com:2222/acme/packs.git".to_owned(),
				dir: Some("ui".to_owned()),
				reference: "abc123".to_owned(),
			}
		);
	}
}
