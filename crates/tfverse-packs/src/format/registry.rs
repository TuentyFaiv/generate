//! Items in the `registry-item.json` format as packs: the item and the items it requires become pieces
//! of one pack, with files placed and imports rewritten for the project's `registry` config.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write;
use std::path::Path;

use serde::Deserialize;
use tfverse_core::model::Digest;
use tfverse_core::workspace::RegistryConfig;

use crate::SourceError;

const MAX_ITEMS: usize = 48;

/// One registry item, as `registry-item.json` describes it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
	name: String,
	#[serde(default)]
	description: Option<String>,
	#[serde(default)]
	dependencies: Vec<String>,
	#[serde(default)]
	dev_dependencies: Vec<String>,
	#[serde(default)]
	registry_dependencies: Vec<String>,
	#[serde(default)]
	files: Vec<ItemFile>,
	#[serde(default)]
	css_vars: Option<serde_json::Value>,
	#[serde(default)]
	css: Option<serde_json::Value>,
	#[serde(default)]
	tailwind: Option<serde_json::Value>,
	#[serde(default)]
	env_vars: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ItemFile {
	path: String,
	#[serde(default)]
	content: Option<String>,
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	target: Option<String>,
}

/// Where a project puts registry items and how their imports read, from its `registry` config.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
	server_components: bool,
	aliases: Aliases,
	namespaces: BTreeMap<String, String>,
	/// Folder that `@/` points to, e.g. `src/`, or empty for the project root.
	base: String,
	/// The stylesheet for CSS variables, when it exists.
	css: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Aliases {
	components: String,
	ui: String,
	lib: String,
	utils: String,
	hooks: String,
}

impl Project {
	/// The project at `root` with its `registry` config; unset aliases use the format's defaults.
	#[must_use]
	pub fn new(root: &Path, config: &RegistryConfig) -> Self {
		let set = &config.aliases;
		let components = set.components.clone().unwrap_or_else(|| "@/components".to_owned());
		let lib = set.lib.clone().unwrap_or_else(|| "@/lib".to_owned());
		let aliases = Aliases {
			ui: set.ui.clone().unwrap_or_else(|| format!("{components}/ui")),
			utils: set.utils.clone().unwrap_or_else(|| format!("{lib}/utils")),
			hooks: set.hooks.clone().unwrap_or_else(|| "@/hooks".to_owned()),
			components,
			lib,
		};
		Self {
			css: config
				.css
				.as_ref()
				.filter(|css| root.join(css.as_str()).is_file())
				.map(ToString::to_string),
			server_components: config.server_components,
			aliases,
			namespaces: config.namespaces.clone(),
			base: if root.join("src").is_dir() {
				"src/".to_owned()
			} else {
				String::new()
			},
		}
	}

	/// The URL of an item reference: an HTTPS URL, `@namespace/name` from the config, or a bare name
	/// next to the item that requires it.
	fn url(&self, reference: &str, beside: Option<&str>) -> Result<String, SourceError> {
		if reference.starts_with("https://") {
			return Ok(reference.to_owned());
		}
		if let Some((namespace, name)) = reference.split_once('/').filter(|_| reference.starts_with('@')) {
			let template = self.namespaces.get(namespace).ok_or_else(|| {
				SourceError::Registry(format!(
					"`{namespace}` is not in `registry.namespaces` of the project config"
				))
			})?;
			let url = template.replace("{name}", name);
			return if url.starts_with("https://") {
				Ok(url)
			} else {
				Err(SourceError::Registry(format!(
					"namespace `{namespace}` is not an HTTPS URL"
				)))
			};
		}
		match beside.and_then(|url| url.rsplit_once('/')) {
			Some((folder, _)) if is_name(reference) => Ok(format!("{folder}/{reference}.json")),
			_ => Err(SourceError::Registry(format!(
				"`{reference}` must be an HTTPS URL or `@namespace/name`"
			))),
		}
	}

	/// The project folder an alias such as `@/components/ui` points to, with a trailing `/`.
	fn folder(&self, alias: &str) -> String {
		let rest = alias
			.strip_prefix("@/")
			.or_else(|| alias.strip_prefix("~/"))
			.unwrap_or(alias);
		format!("{}{}/", self.base, rest.trim_matches('/'))
	}
}

/// The pack files for the item at `reference` and every item it requires, fetched with `get`.
pub(crate) fn pack_files(
	reference: &str,
	project: &Project,
	get: &dyn Fn(&str) -> Result<Vec<u8>, SourceError>,
) -> Result<BTreeMap<String, Vec<u8>>, SourceError> {
	let fetched = fetch(reference, project, get)?;
	let mut files = BTreeMap::new();
	let mut writes = BTreeSet::new();
	let mut skipped = BTreeSet::new();
	let mut has_dependencies = false;
	for (item, url) in fetched.items.iter().zip(&fetched.urls) {
		let (mut piece, depends) = piece_manifest(item, url, &fetched.names, project);
		has_dependencies |= depends;
		let css_applied = match (&item.css_vars, &project.css) {
			(Some(vars), Some(css)) => {
				piece.push_str(&css_var_patches(vars, css));
				writes.insert(css.clone());
				true
			}
			_ => false,
		};
		files.insert(format!("pieces/{}/piece.toml", item.name), piece.into_bytes());
		for file in &item.files {
			let Some(content) = &file.content else {
				continue;
			};
			let target = target(project, file)?;
			let content = rewrite_imports(content, &project.aliases);
			let content = if project.server_components {
				content
			} else {
				drop_use_client(&content)
			};
			writes.insert(target.clone());
			files.insert(format!("pieces/{}/files/{target}", item.name), content.into_bytes());
		}
		skipped.extend(
			[
				("cssVars", item.css_vars.is_some() && !css_applied),
				("css", item.css.is_some()),
				("tailwind", item.tailwind.is_some()),
				("envVars", item.env_vars.is_some()),
			]
			.into_iter()
			.filter(|(_, present)| *present)
			.map(|(field, _)| format!("`{field}` of `{}`", item.name)),
		);
	}
	let main = &fetched.items[0];
	let mut description = main
		.description
		.clone()
		.unwrap_or_else(|| format!("registry item `{}`", main.name));
	if !skipped.is_empty() {
		let _ = write!(
			description,
			"\n\nNot applied by tfverse yet; add them to your CSS, Tailwind config or .env yourself: {}.",
			skipped.into_iter().collect::<Vec<_>>().join(", ")
		);
	}
	let write_paths: Vec<String> = writes.iter().map(|path| quote(path)).collect();
	let manifest = format!(
		"schema_version = 1\nname = {}\nversion = \"{}\"\ndescription = {}\n\n[permissions]\nwrite = [{}]\ndependencies = {has_dependencies}\n",
		quote(&format!("registry-{}", main.name)),
		fetched.version,
		quote(&description),
		write_paths.join(", ")
	);
	files.insert("pack.toml".to_owned(), manifest.into_bytes());
	Ok(files)
}

/// Every item reached from one reference, in the order fetched.
struct Fetched {
	items: Vec<Item>,
	/// The URL each item came from.
	urls: Vec<String>,
	/// The item name each fetched URL served.
	names: BTreeMap<String, String>,
	/// Derived from everything fetched, so it changes exactly when the content does.
	version: String,
}

fn fetch(
	reference: &str,
	project: &Project,
	get: &dyn Fn(&str) -> Result<Vec<u8>, SourceError>,
) -> Result<Fetched, SourceError> {
	let mut queue: VecDeque<(String, Option<String>)> = VecDeque::from([(reference.to_owned(), None)]);
	let mut seen = BTreeSet::new();
	let mut items: Vec<Item> = Vec::new();
	let mut urls = Vec::new();
	let mut names = BTreeMap::new();
	let mut bytes_seen = Vec::new();
	while let Some((next, beside)) = queue.pop_front() {
		let url = project.url(&next, beside.as_deref())?;
		if !seen.insert(url.clone()) {
			continue;
		}
		if seen.len() > MAX_ITEMS {
			return Err(SourceError::Registry(format!("more than {MAX_ITEMS} registry items")));
		}
		let bytes = get(&url)?;
		let item: Item =
			serde_json::from_slice(&bytes).map_err(|error| SourceError::Registry(format!("`{url}`: {error}")))?;
		if !is_name(&item.name) {
			return Err(SourceError::Registry(format!(
				"`{url}`: item name `{}` is not a plain name",
				item.name
			)));
		}
		names.insert(url.clone(), item.name.clone());
		if items.iter().all(|known| known.name != item.name) {
			queue.extend(
				item
					.registry_dependencies
					.iter()
					.map(|dependency| (dependency.clone(), Some(url.clone()))),
			);
			bytes_seen.push(bytes);
			urls.push(url);
			items.push(item);
		}
	}
	let version = format!("0.0.0-h{}", &Digest::of(&bytes_seen.concat()).hex()[..12]);
	Ok(Fetched {
		items,
		urls,
		names,
		version,
	})
}

/// The `piece.toml` of one item, and whether it lists packages.
fn piece_manifest(item: &Item, url: &str, names: &BTreeMap<String, String>, project: &Project) -> (String, bool) {
	let mut piece = format!("schema_version = 1\nname = {}\ntarget = \"\"\n", quote(&item.name));
	if let Some(description) = &item.description {
		let _ = writeln!(piece, "description = {}", quote(description));
	}
	let requires: BTreeSet<String> = item
		.registry_dependencies
		.iter()
		.filter_map(|reference| names.get(&project.url(reference, Some(url)).ok()?))
		.filter(|name| **name != item.name)
		.map(|name| quote(name))
		.collect();
	if !requires.is_empty() {
		let _ = writeln!(
			piece,
			"requires = [{}]",
			requires.into_iter().collect::<Vec<_>>().join(", ")
		);
	}
	let mut depends = false;
	for (packages, dev) in [(&item.dependencies, false), (&item.dev_dependencies, true)] {
		for package in packages {
			let (name, version) = split_package(package);
			let _ = write!(
				piece,
				"\n[[dependencies]]\necosystem = \"npm\"\nname = {}\nversion = {}\n",
				quote(name),
				quote(version)
			);
			if dev {
				piece.push_str("dev = true\n");
			}
			depends = true;
		}
	}
	(piece, depends)
}

/// `css_vars` patches for an item's `cssVars`: `light` in `:root`, `dark` in `.dark`, `theme` in `@theme inline`.
fn css_var_patches(vars: &serde_json::Value, css: &str) -> String {
	let mut patches = String::new();
	for (group, selector) in [("light", ":root"), ("dark", ".dark"), ("theme", "@theme inline")] {
		let Some(values) = vars.get(group).and_then(serde_json::Value::as_object) else {
			continue;
		};
		let entries: Vec<String> = values
			.iter()
			.filter_map(|(name, value)| Some(format!("{} = {}", quote(name), quote(value.as_str()?))))
			.collect();
		if entries.is_empty() {
			continue;
		}
		let _ = write!(
			patches,
			"\n[[patches]]\nkind = \"css_vars\"\nfile = {}\nselector = {}\nvars = {{ {} }}\n",
			quote(css),
			quote(selector),
			entries.join(", ")
		);
	}
	patches
}

/// Where a file goes in the project, following the format's rules for each file type.
fn target(project: &Project, file: &ItemFile) -> Result<String, SourceError> {
	let name = file.path.rsplit('/').next().unwrap_or(&file.path);
	let aliases = &project.aliases;
	let path = match &file.target {
		Some(target) if !target.is_empty() => {
			let placeholders = [
				("@components/", &aliases.components),
				("@ui/", &aliases.ui),
				("@lib/", &aliases.lib),
				("@hooks/", &aliases.hooks),
			];
			placeholders
				.iter()
				.find_map(|(placeholder, alias)| {
					target
						.strip_prefix(placeholder)
						.map(|rest| format!("{}{rest}", project.folder(alias)))
				})
				.unwrap_or_else(|| target.trim_start_matches("~/").to_owned())
		}
		_ => {
			let folder = match file.kind.as_str() {
				"registry:ui" => &aliases.ui,
				"registry:lib" => &aliases.lib,
				"registry:hook" => &aliases.hooks,
				"registry:component" | "registry:block" => &aliases.components,
				kind => {
					return Err(SourceError::Registry(format!(
						"`{}` has type `{kind}` but no `target`",
						file.path
					)));
				}
			};
			format!("{}{name}", project.folder(folder))
		}
	};
	let valid = !path.is_empty()
		&& !path.starts_with('/')
		&& path
			.split('/')
			.all(|part| !part.is_empty() && part != "." && part != "..");
	if valid {
		Ok(path)
	} else {
		Err(SourceError::Registry(format!(
			"`{}` targets an unsafe path `{path}`",
			file.path
		)))
	}
}

/// Points quoted `@/…` imports, from the registry or the format's default aliases, at the project's aliases.
fn rewrite_imports(content: &str, aliases: &Aliases) -> String {
	let mut output = String::with_capacity(content.len());
	let mut rest = content;
	while let Some(at) = [rest.find("\"@/"), rest.find("'@/")].into_iter().flatten().min() {
		let quote = &rest[at..=at];
		output.push_str(&rest[..=at]);
		rest = &rest[at + 1..];
		let end = rest.find(quote).filter(|end| !rest[..*end].contains('\n'));
		let Some(end) = end else {
			continue;
		};
		let specifier = &rest[..end];
		output.push_str(&rewrite_specifier(specifier, aliases).unwrap_or_else(|| specifier.to_owned()));
		rest = &rest[end..];
	}
	output.push_str(rest);
	output
}

fn rewrite_specifier(specifier: &str, aliases: &Aliases) -> Option<String> {
	let path = match specifier.strip_prefix("@/registry/") {
		Some(registry) => registry.split_once('/')?.1.to_owned(),
		None => specifier.strip_prefix("@/")?.to_owned(),
	};
	let (kind, name) = path.split_once('/')?;
	Some(match (kind, name) {
		("lib", "utils") => aliases.utils.clone(),
		("lib", _) => format!("{}/{name}", aliases.lib),
		("hooks", _) => format!("{}/{name}", aliases.hooks),
		("ui", _) => format!("{}/{name}", aliases.ui),
		("components", rest) => match rest.strip_prefix("ui/") {
			Some(ui) => format!("{}/{ui}", aliases.ui),
			None => format!("{}/{rest}", aliases.components),
		},
		_ => return None,
	})
}

/// Removes a leading `"use client"` directive, which only React Server Components projects need.
fn drop_use_client(content: &str) -> String {
	let trimmed = content.trim_start();
	for directive in ["\"use client\"", "'use client'"] {
		if let Some(rest) = trimmed.strip_prefix(directive) {
			return rest.trim_start_matches(';').trim_start_matches(['\r', '\n']).to_owned();
		}
	}
	content.to_owned()
}

/// `name` and version of an npm dependency such as `@scope/pkg@^1.2`; `latest` when none is given.
fn split_package(package: &str) -> (&str, &str) {
	match package.rfind('@') {
		Some(at) if at > 0 => (&package[..at], &package[at + 1..]),
		_ => (package, "latest"),
	}
}

fn quote(text: &str) -> String {
	let mut quoted = String::with_capacity(text.len() + 2);
	quoted.push('"');
	for c in text.chars() {
		match c {
			'"' => quoted.push_str("\\\""),
			'\\' => quoted.push_str("\\\\"),
			'\n' => quoted.push_str("\\n"),
			'\r' => quoted.push_str("\\r"),
			'\t' => quoted.push_str("\\t"),
			c if c.is_control() => {
				let _ = write!(quoted, "\\u{:04X}", u32::from(c));
			}
			c => quoted.push(c),
		}
	}
	quoted.push('"');
	quoted
}

fn is_name(text: &str) -> bool {
	(1..=56).contains(&text.len())
		&& text.starts_with(|c: char| c.is_ascii_lowercase())
		&& text
			.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use tfverse_core::RelPath;
	use tfverse_core::workspace::RegistryAliases;

	pub(crate) const BUTTON: &str = r#"{"name":"button","dependencies":["cn","radix-ui@^1.4"],"registryDependencies":["utils"],"files":[{"path":"registry/default/ui/button.tsx","type":"registry:ui","content":"\"use client\"\n\nimport { cn } from \"@/registry/default/lib/utils\"\nimport { useX } from '@/hooks/use-x'\nexport const label = \"@/not/an/alias\"\n"}],"type":"registry:ui"}"#;
	pub(crate) const UTILS: &str = r#"{"name":"utils","dependencies":["cn"],"files":[{"path":"registry/default/lib/utils.ts","type":"registry:lib","content":"export { cn } from \"cn\"\n"}],"cssVars":{"light":{"x":"1"}},"type":"registry:lib"}"#;
	pub(crate) const BUTTON_URL: &str = "https://registry.example/r/button.json";

	pub(crate) fn serve(url: &str) -> Result<Vec<u8>, SourceError> {
		match url {
			"https://registry.example/r/button.json" => Ok(BUTTON.as_bytes().to_vec()),
			"https://registry.example/r/utils.json" => Ok(UTILS.as_bytes().to_vec()),
			_ => Err(SourceError::NotFound(url.to_owned())),
		}
	}

	fn project(config: &RegistryConfig, src: bool) -> (tempfile::TempDir, Project) {
		let dir = tempfile::tempdir().unwrap();
		if src {
			std::fs::create_dir(dir.path().join("src")).unwrap();
		}
		let project = Project::new(dir.path(), config);
		(dir, project)
	}

	fn text(files: &BTreeMap<String, Vec<u8>>, path: &str) -> String {
		String::from_utf8(files[path].clone()).unwrap()
	}

	#[test]
	fn pack_files_should_place_files_and_require_items_beside_it() {
		let (_dir, project) = project(&RegistryConfig::default(), true);

		let files = pack_files(BUTTON_URL, &project, &serve).unwrap();

		assert_eq!(
			files.keys().collect::<Vec<_>>(),
			[
				"pack.toml",
				"pieces/button/files/src/components/ui/button.tsx",
				"pieces/button/piece.toml",
				"pieces/utils/files/src/lib/utils.ts",
				"pieces/utils/piece.toml",
			]
		);
		let piece = text(&files, "pieces/button/piece.toml");
		assert!(piece.contains("requires = [\"utils\"]"), "{piece}");
		assert!(piece.contains("name = \"radix-ui\"\nversion = \"^1.4\""), "{piece}");
		assert!(piece.contains("name = \"cn\"\nversion = \"latest\""), "{piece}");
	}

	#[test]
	fn pack_files_should_rewrite_imports_for_the_projects_aliases() {
		let config = RegistryConfig {
			server_components: true,
			aliases: RegistryAliases {
				components: Some("~/ui-kit".to_owned()),
				utils: Some("~/lib/cn".to_owned()),
				hooks: Some("~/hooks".to_owned()),
				..RegistryAliases::default()
			},
			..RegistryConfig::default()
		};
		let (_dir, project) = project(&config, false);

		let files = pack_files(BUTTON_URL, &project, &serve).unwrap();

		assert_eq!(
			text(&files, "pieces/button/files/ui-kit/ui/button.tsx"),
			"\"use client\"\n\nimport { cn } from \"~/lib/cn\"\nimport { useX } from '~/hooks/use-x'\nexport const label = \"@/not/an/alias\"\n"
		);
	}

	#[test]
	fn pack_files_should_drop_use_client_outside_server_components() {
		let (_dir, project) = project(&RegistryConfig::default(), false);

		let files = pack_files(BUTTON_URL, &project, &serve).unwrap();

		assert!(text(&files, "pieces/button/files/components/ui/button.tsx").starts_with("import { cn }"));
	}

	#[test]
	fn pack_files_should_name_what_it_did_not_apply() {
		let (_dir, project) = project(&RegistryConfig::default(), false);

		let manifest = text(&pack_files(BUTTON_URL, &project, &serve).unwrap(), "pack.toml");

		assert!(manifest.contains("name = \"registry-button\""), "{manifest}");
		assert!(manifest.contains("`cssVars` of `utils`"), "{manifest}");
		assert!(manifest.contains("version = \"0.0.0-h"), "{manifest}");
	}

	#[test]
	fn css_vars_should_become_patches_when_the_stylesheet_exists() {
		let config = RegistryConfig {
			css: Some(RelPath::new("src/app.css").unwrap()),
			..RegistryConfig::default()
		};
		let dir = tempfile::tempdir().unwrap();
		std::fs::create_dir(dir.path().join("src")).unwrap();
		std::fs::write(dir.path().join("src/app.css"), ":root {}\n").unwrap();
		let project = Project::new(dir.path(), &config);

		let files = pack_files(BUTTON_URL, &project, &serve).unwrap();

		let utils = text(&files, "pieces/utils/piece.toml");
		assert!(
			utils.contains("kind = \"css_vars\"\nfile = \"src/app.css\"\nselector = \":root\"\nvars = { \"x\" = \"1\" }"),
			"{utils}"
		);
		let manifest = text(&files, "pack.toml");
		assert!(manifest.contains("\"src/app.css\""), "{manifest}");
		assert!(!manifest.contains("Not applied"), "{manifest}");
	}

	#[test]
	fn project_url_should_use_namespaces_and_refuse_bare_names_at_the_top() {
		let config = RegistryConfig {
			namespaces: BTreeMap::from([
				("@acme".to_owned(), "https://acme.dev/r/{name}.json".to_owned()),
				("@plain".to_owned(), "http://x/{name}".to_owned()),
			]),
			..RegistryConfig::default()
		};
		let (_dir, project) = project(&config, false);

		assert_eq!(project.url("@acme/card", None).unwrap(), "https://acme.dev/r/card.json");
		assert_eq!(
			project.url("utils", Some("https://acme.dev/r/card.json")).unwrap(),
			"https://acme.dev/r/utils.json"
		);
		assert!(project.url("@plain/card", None).is_err());
		assert!(project.url("@other/card", None).is_err());
		assert!(project.url("card", None).is_err());
	}

	#[test]
	fn target_should_refuse_paths_outside_the_project() {
		let (_dir, project) = project(&RegistryConfig::default(), false);
		let file = ItemFile {
			path: "x".to_owned(),
			content: Some(String::new()),
			kind: "registry:file".to_owned(),
			target: Some("../../.bashrc".to_owned()),
		};

		assert!(target(&project, &file).is_err());
	}

	#[test]
	fn rewrite_imports_should_ignore_apostrophes_in_text() {
		let (_dir, project) = project(&RegistryConfig::default(), false);
		let aliases = Aliases {
			ui: "~/ui".to_owned(),
			..project.aliases
		};

		let rewritten = rewrite_imports(
			"// don't\nimport { B } from \"@/registry/x/ui/button\"\n// it's\n",
			&aliases,
		);

		assert_eq!(rewritten, "// don't\nimport { B } from \"~/ui/button\"\n// it's\n");
	}

	#[test]
	fn split_package_should_keep_scopes() {
		assert_eq!(
			split_package("@radix-ui/react-slot@1.2"),
			("@radix-ui/react-slot", "1.2")
		);
		assert_eq!(
			split_package("@radix-ui/react-slot"),
			("@radix-ui/react-slot", "latest")
		);
	}
}
