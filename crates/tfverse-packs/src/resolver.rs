//! Turns a source string into a loaded, verified pack.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tfverse_core::Pack;
use tfverse_core::model::Digest;
use tfverse_core::workspace::{Lockfile, ProjectConfig, RegistryConfig, UserConfig};

use crate::format::{self, Project, pack_files, unpack, unpack_tar};
use crate::signing::Trust;
use crate::spec::github_api_url;
use crate::store::load_official;
use crate::transport::{self, Credentials, Fetch, Git, Header, NoGit, Offline};
use crate::{Cache, PackSpec, SourceError};

const GITHUB_HOST: &str = "github.com";

const PACK_MANIFEST: &str = "pack.toml";
const COLLECTION_DIR: &str = "packs";

/// Loads packs from every kind of source.
#[derive(Clone, Debug)]
pub struct Resolver {
	root: PathBuf,
	namespaces: BTreeMap<String, String>,
	fetch: Arc<dyn Fetch>,
	git: Arc<dyn Git>,
	credentials: Credentials,
	trust: Trust,
	registry: RegistryConfig,
	cache: Option<Cache>,
}

impl Resolver {
	/// A resolver for the project at `root`, downloading over HTTPS into the user's cache.
	///
	/// Without the `remote` feature it reaches no network, so only official, folder and cached packs load.
	#[must_use]
	pub fn new(root: impl Into<PathBuf>) -> Self {
		Self {
			root: root.into(),
			namespaces: BTreeMap::new(),
			fetch: default_fetch(),
			git: default_git(),
			credentials: Credentials::default(),
			trust: Trust::default(),
			registry: RegistryConfig::default(),
			cache: Cache::user(),
		}
	}

	/// Login headers for private sources, from the user config.
	#[must_use]
	pub fn credentials(mut self, credentials: Credentials) -> Self {
		self.credentials = credentials;
		self
	}

	/// Reaches no network: remote sources load only from the cache.
	#[must_use]
	pub fn offline(self) -> Self {
		self.fetch(Arc::new(Offline)).git(Arc::new(NoGit))
	}

	/// A resolver for the project at `root`: its namespaces and registry config, and the user's
	/// credentials and trusted keys.
	pub fn for_project(
		root: impl Into<PathBuf>,
		project: &ProjectConfig,
		user: &UserConfig,
	) -> Result<Self, SourceError> {
		Ok(
			Self::new(root)
				.namespaces(project.sources.clone())
				.registry(project.registry.clone())
				.credentials(Credentials::new(user.auth.clone()))
				.trust(Trust::new(&user.signing)?),
		)
	}

	/// The pack versions `lock` pins, from the cache or their source; official packs are left out,
	/// since they ship with tfverse. Packs that cannot be loaded are returned with the reason.
	#[must_use]
	pub fn pinned(&self, lock: &Lockfile) -> (Vec<Pack>, Vec<(String, SourceError)>) {
		let mut packs = Vec::new();
		let mut failed = Vec::new();
		for record in &lock.packs {
			let Some(source) = &record.source else {
				continue;
			};
			if matches!(source.parse::<PackSpec>(), Ok(PackSpec::Official(_))) {
				continue;
			}
			match self.resolve(source, Some(record.digest)) {
				Ok(pack) => packs.push(pack),
				Err(error) => failed.push((source.clone(), error)),
			}
		}
		(packs, failed)
	}

	/// Places `registry:` items as the project's `registry` config says.
	#[must_use]
	pub fn registry(mut self, registry: RegistryConfig) -> Self {
		self.registry = registry;
		self
	}

	/// Checks signatures against `trust`, from the user config.
	#[must_use]
	pub fn trust(mut self, trust: Trust) -> Self {
		self.trust = trust;
		self
	}

	/// Uses `git` for `git:` sources.
	#[must_use]
	pub fn git(mut self, git: Arc<dyn Git>) -> Self {
		self.git = git;
		self
	}

	/// Namespaces from the project config, e.g. `"@acme" = "gh:acme/packs@v1.2.0"`.
	#[must_use]
	pub fn namespaces(mut self, namespaces: BTreeMap<String, String>) -> Self {
		self.namespaces = namespaces;
		self
	}

	/// Uses `fetch` for downloads.
	#[must_use]
	pub fn fetch(mut self, fetch: Arc<dyn Fetch>) -> Self {
		self.fetch = fetch;
		self
	}

	/// Uses `cache`, or none.
	#[must_use]
	pub fn cache(mut self, cache: Option<Cache>) -> Self {
		self.cache = cache;
		self
	}

	/// Loads the pack at `source`.
	///
	/// `pinned` is the hash the lockfile recorded for it: a cached copy is used without downloading,
	/// and a pack whose files differ is refused. Official packs are exempt: they ship with tfverse, so
	/// a newer tfverse brings their next version.
	pub fn resolve(&self, source: &str, pinned: Option<Digest>) -> Result<Pack, SourceError> {
		let spec: PackSpec = source.parse()?;
		let pack = match &spec {
			PackSpec::Official(name) => {
				load_official(name).ok_or_else(|| SourceError::NotFound(format!("no official pack `{name}`")))??
			}
			PackSpec::Path(path) => {
				let pack = Pack::load(&self.root.join(path))?;
				if path.is_absolute() {
					pack
				} else {
					pack.with_source(spec.to_string())
				}
			}
			PackSpec::GitHub { dir, .. } | PackSpec::Git { dir, .. } => self.remote(&spec, dir.as_deref(), None, pinned)?,
			PackSpec::Tarball(_) | PackSpec::Registry(_) => self.remote(&spec, None, None, pinned)?,
			PackSpec::Alias { namespace, pack } => {
				let target = self
					.namespaces
					.get(namespace)
					.ok_or_else(|| SourceError::UnknownNamespace(namespace.clone()))?;
				let inner: PackSpec = target.parse()?;
				match &inner {
					PackSpec::GitHub { dir, .. } | PackSpec::Git { dir, .. } => {
						self.remote(&inner, dir.as_deref(), Some(pack), pinned)?
					}
					PackSpec::Tarball(_) => self.remote(&inner, None, Some(pack), pinned)?,
					PackSpec::Path(path) => Pack::load(&collection_member(&self.root.join(path), pack))?,
					PackSpec::Official(_) | PackSpec::Registry(_) | PackSpec::Alias { .. } => {
						return Err(SourceError::NotFound(format!(
							"namespace `{namespace}` must point to a folder, `gh:`, `git:` or `https://` source"
						)));
					}
				}
				.with_source(spec.to_string())
			}
		};
		let pack = if matches!(spec, PackSpec::Official(_)) {
			pack
		} else {
			self.trust.verify(pack)?
		};
		match pinned {
			Some(expected) if expected != pack.digest() && !matches!(spec, PackSpec::Official(_)) => {
				Err(SourceError::Changed {
					spec: spec.to_string(),
					expected,
					actual: pack.digest(),
				})
			}
			_ => Ok(pack),
		}
	}

	fn remote(
		&self,
		spec: &PackSpec,
		dir: Option<&str>,
		member: Option<&str>,
		pinned: Option<Digest>,
	) -> Result<Pack, SourceError> {
		if let (Some(cache), Some(digest)) = (&self.cache, pinned)
			&& let Some(pack) = cache.get(&digest)?
		{
			return Ok(pack.with_source(spec.to_string()));
		}
		let mut files = strip_common_root(self.download(spec)?);
		if let Some(dir) = dir {
			files = format::subtree(files, dir);
		}
		if let Some(member) = member {
			files = select_member(files, member)?;
		}
		if !files.contains_key(PACK_MANIFEST) {
			return Err(SourceError::NotFound(format!("`{spec}` has no `{PACK_MANIFEST}`")));
		}
		let pack = Pack::from_files(files.clone())?;
		if let Some(cache) = &self.cache {
			cache.put(&pack.digest(), &files)?;
		}
		Ok(pack.with_source(spec.to_string()))
	}
}

#[cfg(feature = "remote")]
fn default_fetch() -> Arc<dyn Fetch> {
	Arc::new(crate::transport::HttpsFetch::default())
}

#[cfg(not(feature = "remote"))]
fn default_fetch() -> Arc<dyn Fetch> {
	Arc::new(Offline)
}

#[cfg(feature = "remote")]
fn default_git() -> Arc<dyn Git> {
	Arc::new(crate::transport::SystemGit::default())
}

#[cfg(not(feature = "remote"))]
fn default_git() -> Arc<dyn Git> {
	Arc::new(NoGit)
}

impl Resolver {
	/// Every file of a remote source.
	fn download(&self, spec: &PackSpec) -> Result<BTreeMap<String, Vec<u8>>, SourceError> {
		match spec {
			PackSpec::Git { url, reference, .. } => Ok(unpack_tar(&self.git.archive(url, reference)?, false)?),
			PackSpec::GitHub { .. } => {
				let headers = self.credentials.headers(GITHUB_HOST)?;
				let url = if headers.is_empty() {
					spec.archive_url()
				} else {
					github_api_url(spec)
				};
				let url = url.ok_or_else(|| SourceError::NotFound(format!("`{spec}` is not remote")))?;
				Ok(unpack(&self.get(&url, GITHUB_HOST, &headers)?, false)?)
			}
			PackSpec::Tarball(url) => {
				let host = transport::host(url).unwrap_or_default();
				let headers = self.credentials.headers(&host)?;
				Ok(unpack(&self.get(url, &host, &headers)?, false)?)
			}
			PackSpec::Registry(item) => {
				let project = Project::new(&self.root, &self.registry);
				pack_files(item, &project, &|url| {
					let host = transport::host(url).unwrap_or_default();
					let headers = self.credentials.headers(&host)?;
					self.get(url, &host, &headers)
				})
			}
			PackSpec::Official(_) | PackSpec::Path(_) | PackSpec::Alias { .. } => {
				Err(SourceError::NotFound(format!("`{spec}` is not remote")))
			}
		}
	}

	/// Downloads `url`, suggesting credentials when a host without them refuses.
	fn get(&self, url: &str, host: &str, headers: &[Header]) -> Result<Vec<u8>, SourceError> {
		self.fetch.get(url, headers).map_err(|mut error| {
			let refused = ["HTTP 401", "HTTP 403", "HTTP 404"]
				.iter()
				.any(|status| error.reason.contains(status));
			if refused && headers.is_empty() {
				let hint = if self.credentials.configured(host) {
					format!("; the token variable for `{host}` in your user config is not set")
				} else {
					format!(
						"; if the source is private, add `[auth.\"{host}\"]` with `env = \"YOUR_TOKEN_VARIABLE\"` to your user config"
					)
				};
				error.reason += &hint;
			}
			SourceError::Fetch(error)
		})
	}
}

/// Removes the single top folder that release archives wrap their content in.
fn strip_common_root(files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
	let mut roots = files.keys().map(|path| path.split_once('/').map(|(root, _)| root));
	let common = match roots.next() {
		Some(Some(first)) if roots.all(|root| root == Some(first)) => Some(first.to_owned()),
		_ => None,
	};
	match common {
		Some(root) if !files.contains_key(PACK_MANIFEST) => format::subtree(files, &root),
		_ => files,
	}
}

/// The pack `member` of a collection: the root pack if it has that name, else `packs/<member>/`.
fn select_member(files: BTreeMap<String, Vec<u8>>, member: &str) -> Result<BTreeMap<String, Vec<u8>>, SourceError> {
	let nested = format!("{COLLECTION_DIR}/{member}/{PACK_MANIFEST}");
	if files.contains_key(&nested) {
		return Ok(format::subtree(files, &format!("{COLLECTION_DIR}/{member}")));
	}
	if files.contains_key(PACK_MANIFEST) {
		return Ok(files);
	}
	Err(SourceError::NotFound(format!("no pack `{member}` in this namespace")))
}

fn collection_member(dir: &Path, member: &str) -> PathBuf {
	let nested = dir.join(COLLECTION_DIR).join(member);
	if nested.join(PACK_MANIFEST).is_file() {
		nested
	} else {
		dir.to_path_buf()
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Mutex;

	use super::*;
	use crate::format::archive_tests::tarball;
	use crate::transport::{FetchError, SystemGit};

	const PACK: &str = "schema_version = 1\nname = \"ui\"\nversion = \"1.0.0\"\n";
	const PIECE: &str = "schema_version = 1\nname = \"atom\"\ntarget = \"src\"\n";

	/// Serves one archive and records every URL asked for.
	#[derive(Debug, Default)]
	struct Served {
		body: Vec<u8>,
		asked: Mutex<Vec<String>>,
	}

	impl Fetch for Served {
		fn get(&self, url: &str, headers: &[Header]) -> Result<Vec<u8>, FetchError> {
			let sent: Vec<&str> = headers.iter().map(Header::name).collect();
			self
				.asked
				.lock()
				.map(|mut asked| asked.push(format!("{url} {}", sent.join(",")).trim_end().to_owned()))
				.ok();
			Ok(self.body.clone())
		}
	}

	fn served(files: &[(&str, &str)]) -> Arc<Served> {
		Arc::new(Served {
			body: tarball(files),
			..Served::default()
		})
	}

	fn pack_files(prefix: &str, atom: &str) -> Vec<(String, String)> {
		vec![
			(format!("{prefix}pack.toml"), PACK.to_owned()),
			(format!("{prefix}pieces/atom/piece.toml"), PIECE.to_owned()),
			(format!("{prefix}pieces/atom/files/atom.ts"), atom.to_owned()),
		]
	}

	fn tar(files: &[(String, String)]) -> Arc<Served> {
		let borrowed: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
		served(&borrowed)
	}

	fn resolver(fetch: Arc<Served>, cache: &tempfile::TempDir) -> Resolver {
		Resolver::new(cache.path())
			.fetch(fetch)
			.cache(Some(Cache::new(cache.path().join("cache"))))
	}

	/// Answers every download with HTTP 404, as GitHub does for private repositories.
	#[derive(Debug)]
	struct NotFound;

	impl Fetch for NotFound {
		fn get(&self, url: &str, _headers: &[Header]) -> Result<Vec<u8>, FetchError> {
			Err(FetchError {
				url: url.to_owned(),
				reason: "the server answered HTTP 404".to_owned(),
			})
		}
	}

	/// Serves a local repository through the real git, whatever URL is asked for.
	#[derive(Debug)]
	struct LocalGit(PathBuf);

	impl Git for LocalGit {
		fn archive(&self, _url: &str, reference: &str) -> Result<Vec<u8>, FetchError> {
			SystemGit::allowing_local().archive(&self.0.display().to_string(), reference)
		}
	}

	fn github_token() -> Credentials {
		let auth = tfverse_core::workspace::HostAuth {
			env: "GITHUB_TOKEN".to_owned(),
			header: "Authorization".to_owned(),
			scheme: "Bearer".to_owned(),
		};
		Credentials::new(BTreeMap::from([("github.com".to_owned(), auth)]))
			.with_env(|name| (name == "GITHUB_TOKEN").then(|| "ghp_secret".to_owned()))
	}

	#[test]
	fn github_with_a_token_should_use_the_api_archive_and_send_the_header() {
		let fetch = tar(&pack_files("acme-packs-abc/", "x\n"));
		let dir = tempfile::tempdir().unwrap();

		resolver(fetch.clone(), &dir)
			.credentials(github_token())
			.resolve("gh:acme/packs@v1.0.0", None)
			.unwrap();

		assert_eq!(
			*fetch.asked.lock().unwrap(),
			["https://api.github.com/repos/acme/packs/tarball/v1.0.0 Authorization"]
		);
	}

	#[test]
	fn tarball_should_send_only_its_hosts_credentials() {
		let fetch = tar(&pack_files("ui/", "x\n"));
		let dir = tempfile::tempdir().unwrap();

		resolver(fetch.clone(), &dir)
			.credentials(github_token())
			.resolve("https://example.com/ui-1.0.0.tar.gz", None)
			.unwrap();

		assert_eq!(*fetch.asked.lock().unwrap(), ["https://example.com/ui-1.0.0.tar.gz"]);
	}

	#[test]
	fn a_refused_download_should_suggest_credentials() {
		let dir = tempfile::tempdir().unwrap();

		let error = Resolver::new(dir.path())
			.fetch(Arc::new(NotFound))
			.cache(None)
			.resolve("gh:acme/private@v1.0.0", None)
			.unwrap_err()
			.to_string();

		assert!(error.contains("[auth.\"github.com\"]"), "{error}");
	}

	#[test]
	fn git_source_should_load_a_pack_from_a_repository_folder() {
		let files = pack_files("packs/ui/", "x\n");
		let borrowed: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
		let repo = crate::transport::git_tests::repository(&borrowed);
		let dir = tempfile::tempdir().unwrap();

		let pack = Resolver::new(dir.path())
			.git(Arc::new(LocalGit(repo.path().to_path_buf())))
			.cache(None)
			.resolve("git:git@example.com:acme/packs.git//packs/ui@v1.0.0", None)
			.unwrap();

		assert_eq!(pack.manifest().name.as_str(), "ui");
		assert_eq!(
			pack.source(),
			Some("git:git@example.com:acme/packs.git//packs/ui@v1.0.0")
		);
	}

	fn signing(require: bool) -> Trust {
		Trust::new(&tfverse_core::workspace::SigningConfig {
			require,
			keys: BTreeMap::from([("acme".to_owned(), crate::signing::tests::TRUSTED_KEY.to_owned())]),
			revoked: Vec::new(),
		})
		.unwrap()
	}

	#[test]
	fn a_signed_download_should_carry_its_signer() {
		let mut files = pack_files("packs-1.0.0/", "x\n");
		files.push((
			"packs-1.0.0/pack.minisig".to_owned(),
			crate::signing::tests::TRUSTED_SIG.to_owned(),
		));
		let dir = tempfile::tempdir().unwrap();

		let pack = resolver(tar(&files), &dir)
			.trust(signing(true))
			.resolve("gh:acme/packs@v1.0.0", None)
			.unwrap();

		assert_eq!(pack.signer(), Some("1C3B7AE36BE79191"));
	}

	#[test]
	fn required_signatures_should_refuse_unsigned_downloads_but_not_official_packs() {
		let dir = tempfile::tempdir().unwrap();
		let resolver = resolver(tar(&pack_files("packs-1.0.0/", "x\n")), &dir).trust(signing(true));

		let unsigned = resolver.resolve("gh:acme/packs@v1.0.0", None);

		assert!(matches!(unsigned, Err(SourceError::Signature(_))), "{unsigned:?}");
		assert!(resolver.resolve("ui", None).is_ok());
	}

	#[test]
	fn offline_resolver_should_load_a_pinned_pack_from_the_cache_only() {
		let dir = tempfile::tempdir().unwrap();
		let pinned = resolver(tar(&pack_files("packs-1.0.0/", "x\n")), &dir)
			.resolve("gh:acme/packs@v1.0.0", None)
			.unwrap()
			.digest();
		let offline = resolver(Arc::new(Served::default()), &dir).offline();

		let cached = offline.resolve("gh:acme/packs@v1.0.0", Some(pinned)).unwrap();
		let unpinned = offline.resolve("gh:acme/packs@v1.0.0", None);
		let git = offline.resolve("git:git@example.com:acme/packs.git@v1.0.0", None);

		assert_eq!(cached.digest(), pinned);
		assert!(matches!(unpinned, Err(SourceError::Fetch(_))));
		assert!(matches!(git, Err(SourceError::Fetch(_))));
	}

	#[test]
	fn github_source_should_download_the_tag_and_record_its_source() {
		let fetch = tar(&pack_files("packs-1.0.0/", "x\n"));
		let dir = tempfile::tempdir().unwrap();

		let pack = resolver(fetch.clone(), &dir)
			.resolve("gh:acme/packs@v1.0.0", None)
			.unwrap();

		assert_eq!(pack.manifest().name.as_str(), "ui");
		assert_eq!(pack.source(), Some("gh:acme/packs@v1.0.0"));
		assert_eq!(
			*fetch.asked.lock().unwrap(),
			["https://codeload.github.com/acme/packs/tar.gz/v1.0.0"]
		);
	}

	#[test]
	fn pinned_pack_should_load_from_the_cache_without_downloading() {
		let dir = tempfile::tempdir().unwrap();
		let first = resolver(tar(&pack_files("root/", "x\n")), &dir)
			.resolve("gh:acme/packs@v1.0.0", None)
			.unwrap();
		let offline = Arc::new(Served::default());

		let again = resolver(offline.clone(), &dir)
			.resolve("gh:acme/packs@v1.0.0", Some(first.digest()))
			.unwrap();

		assert_eq!(again.digest(), first.digest());
		assert!(offline.asked.lock().unwrap().is_empty());
	}

	#[test]
	fn pinned_pack_should_be_refused_when_the_source_changed() {
		let dir = tempfile::tempdir().unwrap();
		let pinned = resolver(tar(&pack_files("root/", "x\n")), &dir)
			.resolve("gh:acme/packs@v1.0.0", None)
			.unwrap()
			.digest();
		let other = tempfile::tempdir().unwrap();

		let result =
			resolver(tar(&pack_files("root/", "tampered\n")), &other).resolve("gh:acme/packs@v1.0.0", Some(pinned));

		assert!(matches!(result, Err(SourceError::Changed { .. })), "{result:?}");
	}

	#[test]
	fn namespace_should_pick_the_pack_from_a_collection() {
		let dir = tempfile::tempdir().unwrap();
		let mut files = pack_files("repo-v2/packs/ui/", "x\n");
		files.push(("repo-v2/README.md".to_owned(), "hi".to_owned()));
		let namespaces = BTreeMap::from([("@acme".to_owned(), "gh:acme/packs@v2".to_owned())]);

		let pack = resolver(tar(&files), &dir)
			.namespaces(namespaces)
			.resolve("@acme/ui", None)
			.unwrap();

		assert_eq!(pack.manifest().name.as_str(), "ui");
		assert_eq!(pack.source(), Some("@acme/ui"));
	}

	#[test]
	fn unknown_namespace_should_say_how_to_configure_it() {
		let dir = tempfile::tempdir().unwrap();

		let error = resolver(Arc::new(Served::default()), &dir)
			.resolve("@acme/ui", None)
			.unwrap_err();

		assert!(error.to_string().contains("sources"), "{error}");
	}

	#[test]
	fn official_pack_should_load_when_a_newer_tfverse_changed_it() {
		let dir = tempfile::tempdir().unwrap();
		let pinned = Digest::of(b"an older official ui");

		let pack = resolver(Arc::new(Served::default()), &dir).resolve("ui", Some(pinned));

		assert!(pack.is_ok(), "{pack:?}");
	}

	#[test]
	fn relative_folder_should_be_recorded_and_official_packs_should_load() {
		let dir = tempfile::tempdir().unwrap();
		for (path, text) in pack_files("mine/", "x\n") {
			let file = dir.path().join(path);
			std::fs::create_dir_all(file.parent().unwrap()).unwrap();
			std::fs::write(file, text).unwrap();
		}
		let resolver = resolver(Arc::new(Served::default()), &dir);

		assert_eq!(resolver.resolve("./mine", None).unwrap().source(), Some("path:./mine"));
		assert_eq!(
			resolver.resolve("backend", None).unwrap().source(),
			Some("official:backend")
		);
	}
}
