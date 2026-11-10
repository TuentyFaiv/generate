//! Project model: the detected stack and environment variable names.

use serde::Serialize;
use serde_json::Value as Json;
use std::collections::BTreeSet;

use super::{ProjectConfig, Workspace, WorkspaceError};
use crate::model::{Fact, RelPath, Stack};

/// What tfverse knows about a project.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ProjectModel {
	/// Stack facts.
	pub stack: Stack,
	/// Environment variable names from `.env.example`; values are never read.
	pub env: BTreeSet<String>,
}

/// Detects stack facts for one ecosystem.
pub(crate) trait Detector {
	/// Adds the facts this detector finds.
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError>;
}

/// Every built-in detector.
pub(crate) fn detectors() -> Vec<Box<dyn Detector>> {
	vec![
		Box::new(JavaScript),
		Box::new(Rust),
		Box::new(Python),
		Box::new(Go),
		Box::new(Dart),
		Box::new(Ruby),
		Box::new(Php),
		Box::new(Jvm),
		Box::new(Swift),
	]
}

/// Detects the project model, then applies config overrides.
pub(crate) fn detect(workspace: &Workspace, config: &ProjectConfig) -> Result<ProjectModel, WorkspaceError> {
	let mut stack = Stack::default();
	for detector in detectors() {
		detector.detect(workspace, &mut stack)?;
	}
	for (fact, values) in &config.stack {
		stack.set(*fact, values.iter().cloned());
	}
	Ok(ProjectModel {
		stack,
		env: env_keys(workspace)?,
	})
}

fn read_text(workspace: &Workspace, path: &'static str) -> Result<Option<String>, WorkspaceError> {
	Ok(
		workspace
			.read(&RelPath::constant(path))?
			.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()),
	)
}

fn exists(workspace: &Workspace, path: &'static str) -> Result<bool, WorkspaceError> {
	workspace.exists(&RelPath::constant(path))
}

fn any_exists(workspace: &Workspace, paths: &[&'static str]) -> Result<bool, WorkspaceError> {
	for path in paths {
		if exists(workspace, path)? {
			return Ok(true);
		}
	}
	Ok(false)
}

fn env_keys(workspace: &Workspace) -> Result<BTreeSet<String>, WorkspaceError> {
	let Some(text) = read_text(workspace, ".env.example")? else {
		return Ok(BTreeSet::new());
	};
	Ok(
		text
			.lines()
			.filter_map(|line| {
				let line = line.trim().strip_prefix("export ").unwrap_or(line.trim());
				let (key, _) = line.split_once('=')?;
				let key = key.trim();
				(!key.is_empty() && !key.starts_with('#') && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
					.then(|| key.to_owned())
			})
			.collect(),
	)
}

fn insert_named(stack: &mut Stack, fact: Fact, names: &BTreeSet<String>, table: &[(&str, &str)]) {
	for (name, value) in table {
		if names.contains(*name) {
			stack.insert(fact, value);
		}
	}
}

fn insert_contained(stack: &mut Stack, fact: Fact, text: &str, table: &[(&str, &str)]) {
	for (needle, value) in table {
		if text.contains(needle) {
			stack.insert(fact, value);
		}
	}
}

fn toml_get<'a>(table: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Value> {
	let (first, rest) = path.split_first()?;
	rest.iter().try_fold(table.get(*first)?, |value, key| value.get(*key))
}

fn toml_keys(value: Option<&toml::Value>) -> impl Iterator<Item = &String> {
	value
		.and_then(toml::Value::as_table)
		.into_iter()
		.flat_map(toml::Table::keys)
}

fn toml_strings(value: Option<&toml::Value>) -> impl Iterator<Item = &str> {
	value
		.and_then(toml::Value::as_array)
		.into_iter()
		.flatten()
		.filter_map(toml::Value::as_str)
}

fn json_keys<'a>(manifest: &'a Json, sections: &'a [&str]) -> impl Iterator<Item = String> + 'a {
	sections
		.iter()
		.filter_map(|section| manifest.get(section)?.as_object())
		.flat_map(|deps| deps.keys().cloned())
}

/// Parses JSON with comments and trailing commas, as in `deno.jsonc`.
fn parse_jsonc(text: &str) -> Option<Json> {
	let mut stripped = String::with_capacity(text.len());
	let mut chars = text.chars().peekable();
	let mut in_string = false;
	while let Some(c) = chars.next() {
		if in_string {
			stripped.push(c);
			match c {
				'\\' => stripped.extend(chars.next()),
				'"' => in_string = false,
				_ => {}
			}
			continue;
		}
		match c {
			'"' => {
				in_string = true;
				stripped.push(c);
			}
			'/' if chars.peek() == Some(&'/') => while chars.next_if(|&next| next != '\n').is_some() {},
			'/' if chars.peek() == Some(&'*') => {
				chars.next();
				let mut previous = ' ';
				for next in chars.by_ref() {
					if previous == '*' && next == '/' {
						break;
					}
					previous = next;
				}
			}
			',' => {
				let rest = chars.clone().find(|next| !next.is_whitespace());
				if !matches!(rest, Some('}' | ']')) {
					stripped.push(c);
				}
			}
			_ => stripped.push(c),
		}
	}
	serde_json::from_str(&stripped).ok()
}

struct JavaScript;

const JS_FRAMEWORKS: [(&str, &str); 26] = [
	("react", "react"),
	("next", "next"),
	("svelte", "svelte"),
	("@sveltejs/kit", "sveltekit"),
	("astro", "astro"),
	("vue", "vue"),
	("nuxt", "nuxt"),
	("solid-js", "solid"),
	("@solidjs/start", "solidstart"),
	("@angular/core", "angular"),
	("@builder.io/qwik", "qwik"),
	("@tanstack/react-start", "tanstack-start"),
	("react-router", "react-router"),
	("lit", "lit"),
	("express", "express"),
	("fastify", "fastify"),
	("hono", "hono"),
	("@hono/hono", "hono"),
	("elysia", "elysia"),
	("@nestjs/core", "nestjs"),
	("@fresh/core", "fresh"),
	("react-native", "react-native"),
	("expo", "expo"),
	("electron", "electron"),
	("@tauri-apps/api", "tauri"),
	("preact", "preact"),
];

const JS_STYLING: [(&str, &str); 8] = [
	("tailwindcss", "tailwind"),
	("@stylexjs/stylex", "stylex"),
	("sass", "sass"),
	("styled-components", "styled-components"),
	("@emotion/react", "emotion"),
	("@vanilla-extract/css", "vanilla-extract"),
	("@pandacss/dev", "panda"),
	("unocss", "unocss"),
];

const JS_DATABASES: [(&str, &str); 9] = [
	("prisma", "prisma"),
	("@prisma/client", "prisma"),
	("drizzle-orm", "drizzle"),
	("typeorm", "typeorm"),
	("sequelize", "sequelize"),
	("@mikro-orm/core", "mikro-orm"),
	("kysely", "kysely"),
	("mongoose", "mongoose"),
	("surrealdb", "surrealdb"),
];

const PACKAGE_SECTIONS: [&str; 4] = [
	"dependencies",
	"devDependencies",
	"peerDependencies",
	"optionalDependencies",
];

impl Detector for JavaScript {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let package_text = read_text(workspace, "package.json")?;
		let deno_text = match read_text(workspace, "deno.json")? {
			Some(text) => Some(text),
			None => read_text(workspace, "deno.jsonc")?,
		};
		if package_text.is_none() && deno_text.is_none() {
			return Ok(());
		}
		let package = package_text
			.and_then(|text| serde_json::from_str::<Json>(&text).ok())
			.unwrap_or_default();
		let deno = deno_text.is_some();

		let mut names: BTreeSet<String> = json_keys(&package, &PACKAGE_SECTIONS).collect();
		if let Some(imports) = deno_text
			.as_deref()
			.and_then(parse_jsonc)
			.and_then(|config| config.get("imports")?.as_object().cloned())
		{
			names.extend(imports.values().filter_map(Json::as_str).filter_map(specifier_package));
		}

		insert_named(stack, Fact::Framework, &names, &JS_FRAMEWORKS);
		insert_named(stack, Fact::Styling, &names, &JS_STYLING);
		insert_named(stack, Fact::Database, &names, &JS_DATABASES);

		let typescript = deno || names.contains("typescript") || exists(workspace, "tsconfig.json")?;
		stack.insert(Fact::Language, if typescript { "typescript" } else { "javascript" });

		let declared = package
			.get("packageManager")
			.and_then(Json::as_str)
			.and_then(|value| value.split('@').next())
			.map(str::to_owned);
		let bun = declared.as_deref() == Some("bun")
			|| names.contains("@types/bun")
			|| any_exists(workspace, &["bun.lock", "bun.lockb", "bunfig.toml"])?;
		let runtime = if deno {
			"deno"
		} else if bun {
			"bun"
		} else {
			"node"
		};
		stack.insert(Fact::Runtime, runtime);

		let manager = match declared {
			Some(manager) => manager,
			None if any_exists(workspace, &["bun.lock", "bun.lockb"])? => "bun".to_owned(),
			None if exists(workspace, "pnpm-lock.yaml")? => "pnpm".to_owned(),
			None if exists(workspace, "yarn.lock")? => "yarn".to_owned(),
			None if exists(workspace, "package-lock.json")? => "npm".to_owned(),
			None if deno => "deno".to_owned(),
			None => "npm".to_owned(),
		};
		stack.insert(Fact::PackageManager, &manager);
		Ok(())
	}
}

/// The package name in an `npm:` or `jsr:` specifier, e.g. `jsr:@hono/hono@^4` → `@hono/hono`.
fn specifier_package(specifier: &str) -> Option<String> {
	let rest = specifier
		.strip_prefix("npm:")
		.or_else(|| specifier.strip_prefix("jsr:"))?
		.trim_start_matches('/');
	let (scope, rest) = match rest.strip_prefix('@') {
		Some(scoped) => {
			let (scope, rest) = scoped.split_once('/')?;
			(Some(scope), rest)
		}
		None => (None, rest),
	};
	let name = rest.split(['@', '/']).next().filter(|name| !name.is_empty())?;
	Some(scope.map_or_else(|| name.to_owned(), |scope| format!("@{scope}/{name}")))
}

struct Rust;

const RUST_FRAMEWORKS: [(&str, &str); 13] = [
	("axum", "axum"),
	("actix-web", "actix-web"),
	("rocket", "rocket"),
	("poem", "poem"),
	("warp", "warp"),
	("tauri", "tauri"),
	("leptos", "leptos"),
	("dioxus", "dioxus"),
	("yew", "yew"),
	("bevy", "bevy"),
	("gpui", "gpui"),
	("gpui-kit", "gpui-kit"),
	("gpui-component", "gpui-component"),
];

const RUST_RUNTIMES: [(&str, &str); 1] = [("tokio", "tokio")];

const RUST_DATABASES: [(&str, &str); 4] = [
	("diesel", "diesel"),
	("sea-orm", "seaorm"),
	("sqlx", "sqlx"),
	("surrealdb", "surrealdb"),
];

impl Detector for Rust {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let Some(text) = read_text(workspace, "Cargo.toml")? else {
			return Ok(());
		};
		stack.insert(Fact::Language, "rust");
		stack.insert(Fact::PackageManager, "cargo");
		let Ok(manifest) = toml::from_str::<toml::Table>(&text) else {
			return Ok(());
		};
		let names = cargo_dependencies(&manifest);
		insert_named(stack, Fact::Framework, &names, &RUST_FRAMEWORKS);
		insert_named(stack, Fact::Runtime, &names, &RUST_RUNTIMES);
		insert_named(stack, Fact::Database, &names, &RUST_DATABASES);
		if names.contains("gpui-kit") {
			stack.insert(Fact::Framework, "gpui");
		}
		Ok(())
	}
}

/// Crate names of every dependency, following `package = "…"` renames.
fn cargo_dependencies(manifest: &toml::Table) -> BTreeSet<String> {
	[
		&["dependencies"][..],
		&["dev-dependencies"],
		&["build-dependencies"],
		&["workspace", "dependencies"],
	]
	.into_iter()
	.filter_map(|path| toml_get(manifest, path)?.as_table())
	.flatten()
	.map(|(key, value)| {
		value
			.get("package")
			.and_then(toml::Value::as_str)
			.unwrap_or(key)
			.to_owned()
	})
	.collect()
}

struct Python;

const PYTHON_FRAMEWORKS: [(&str, &str); 11] = [
	("fastapi", "fastapi"),
	("django", "django"),
	("flask", "flask"),
	("litestar", "litestar"),
	("starlette", "starlette"),
	("sanic", "sanic"),
	("aiohttp", "aiohttp"),
	("quart", "quart"),
	("falcon", "falcon"),
	("tornado", "tornado"),
	("pyramid", "pyramid"),
];

const PYTHON_DATABASES: [(&str, &str); 5] = [
	("sqlalchemy", "sqlalchemy"),
	("sqlmodel", "sqlmodel"),
	("tortoise-orm", "tortoise"),
	("peewee", "peewee"),
	("beanie", "beanie"),
];

impl Detector for Python {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let pyproject = read_text(workspace, "pyproject.toml")?;
		let requirements = read_text(workspace, "requirements.txt")?;
		let pipfile = read_text(workspace, "Pipfile")?;
		if pyproject.is_none() && requirements.is_none() && pipfile.is_none() {
			return Ok(());
		}
		stack.insert(Fact::Language, "python");

		let pyproject = pyproject.and_then(|text| toml::from_str::<toml::Table>(&text).ok());
		let mut requirements_list: Vec<String> = requirements
			.iter()
			.flat_map(|text| text.lines())
			.map(str::trim)
			.filter(|line| !line.starts_with(['#', '-']))
			.map(str::to_owned)
			.collect();
		if let Some(pyproject) = &pyproject {
			requirements_list.extend(pyproject_requirements(pyproject));
		}
		if let Some(pipfile) = pipfile.and_then(|text| toml::from_str::<toml::Table>(&text).ok()) {
			for section in ["packages", "dev-packages"] {
				requirements_list.extend(toml_keys(pipfile.get(section)).cloned());
			}
		}
		let names: BTreeSet<String> = requirements_list.iter().filter_map(|line| python_name(line)).collect();
		insert_named(stack, Fact::Framework, &names, &PYTHON_FRAMEWORKS);
		if names.contains("django") {
			stack.insert(Fact::Database, "django-orm");
		}
		insert_named(stack, Fact::Database, &names, &PYTHON_DATABASES);

		let poetry = pyproject
			.as_ref()
			.is_some_and(|pyproject| toml_get(pyproject, &["tool", "poetry"]).is_some());
		let manager = if exists(workspace, "uv.lock")? {
			"uv"
		} else if poetry || exists(workspace, "poetry.lock")? {
			"poetry"
		} else if exists(workspace, "pdm.lock")? {
			"pdm"
		} else if exists(workspace, "Pipfile")? {
			"pipenv"
		} else {
			"pip"
		};
		stack.insert(Fact::PackageManager, manager);
		Ok(())
	}
}

/// Requirements from PEP 621, PEP 735 dependency groups and Poetry tables.
fn pyproject_requirements(pyproject: &toml::Table) -> Vec<String> {
	let mut found: Vec<String> = toml_strings(toml_get(pyproject, &["project", "dependencies"]))
		.map(str::to_owned)
		.collect();
	for table in ["project.optional-dependencies", "dependency-groups"] {
		let path: Vec<&str> = table.split('.').collect();
		if let Some(groups) = toml_get(pyproject, &path).and_then(toml::Value::as_table) {
			found.extend(
				groups
					.values()
					.flat_map(|group| toml_strings(Some(group)))
					.map(str::to_owned),
			);
		}
	}
	for path in [
		&["tool", "poetry", "dependencies"][..],
		&["tool", "poetry", "dev-dependencies"],
	] {
		found.extend(toml_keys(toml_get(pyproject, path)).cloned());
	}
	if let Some(groups) = toml_get(pyproject, &["tool", "poetry", "group"]).and_then(toml::Value::as_table) {
		found.extend(
			groups
				.values()
				.flat_map(|group| toml_keys(group.get("dependencies")))
				.cloned(),
		);
	}
	found
}

/// The normalized (PEP 503) project name at the start of a requirement, e.g. `Django[argon2]>=5` → `django`.
fn python_name(requirement: &str) -> Option<String> {
	let name: String = requirement
		.trim()
		.chars()
		.take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
		.collect();
	if name.is_empty() {
		return None;
	}
	let mut normalized = String::with_capacity(name.len());
	for c in name.chars() {
		if matches!(c, '-' | '_' | '.') {
			if !normalized.ends_with('-') {
				normalized.push('-');
			}
		} else {
			normalized.push(c.to_ascii_lowercase());
		}
	}
	Some(normalized)
}

struct Go;

const GO_FRAMEWORKS: [(&str, &str); 8] = [
	("github.com/gin-gonic/gin", "gin"),
	("github.com/labstack/echo", "echo"),
	("github.com/gofiber/fiber", "fiber"),
	("github.com/go-chi/chi", "chi"),
	("github.com/gorilla/mux", "gorilla-mux"),
	("github.com/beego/beego", "beego"),
	("github.com/kataras/iris", "iris"),
	("github.com/cloudwego/hertz", "hertz"),
];

const GO_DATABASES: [(&str, &str); 2] = [("gorm.io/gorm", "gorm"), ("entgo.io/ent", "ent")];

impl Detector for Go {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let Some(text) = read_text(workspace, "go.mod")? else {
			return Ok(());
		};
		stack.insert(Fact::Language, "go");
		stack.insert(Fact::PackageManager, "go");
		let modules = go_modules(&text);
		for (fact, table) in [
			(Fact::Framework, &GO_FRAMEWORKS[..]),
			(Fact::Database, &GO_DATABASES[..]),
		] {
			for (prefix, value) in table {
				let required = modules.iter().any(|module| {
					module
						.strip_prefix(prefix)
						.is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
				});
				if required {
					stack.insert(fact, value);
				}
			}
		}
		Ok(())
	}
}

/// Module paths in `require` lines and blocks of a `go.mod`.
fn go_modules(text: &str) -> BTreeSet<String> {
	let mut modules = BTreeSet::new();
	let mut in_block = false;
	for line in text.lines() {
		let line = line.split("//").next().unwrap_or_default().trim();
		if in_block {
			if line == ")" {
				in_block = false;
			} else if let Some(module) = line.split_whitespace().next() {
				modules.insert(module.to_owned());
			}
		} else if let Some(rest) = line.strip_prefix("require") {
			let rest = rest.trim();
			if rest.starts_with('(') {
				in_block = true;
			} else if let Some(module) = rest.split_whitespace().next() {
				modules.insert(module.to_owned());
			}
		}
	}
	modules
}

struct Dart;

const DART_DATABASES: [(&str, &str); 5] = [
	("drift", "drift"),
	("isar", "isar"),
	("objectbox", "objectbox"),
	("hive", "hive"),
	("floor", "floor"),
];

impl Detector for Dart {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let Some(text) = read_text(workspace, "pubspec.yaml")? else {
			return Ok(());
		};
		stack.insert(Fact::Language, "dart");
		stack.insert(Fact::PackageManager, "pub");
		if text.lines().any(|line| line.trim() == "sdk: flutter") {
			stack.insert(Fact::Framework, "flutter");
		}
		insert_named(stack, Fact::Database, &pubspec_dependencies(&text), &DART_DATABASES);
		Ok(())
	}
}

/// Package names under `dependencies` and `dev_dependencies` in a `pubspec.yaml`.
fn pubspec_dependencies(text: &str) -> BTreeSet<String> {
	let mut names = BTreeSet::new();
	let mut in_dependencies = false;
	for line in text.lines() {
		if line.trim().is_empty() || line.trim_start().starts_with('#') {
			continue;
		}
		if !line.starts_with([' ', '\t']) {
			in_dependencies = matches!(line.trim_end(), "dependencies:" | "dev_dependencies:");
			continue;
		}
		let nested = line.strip_prefix("  ").filter(|rest| !rest.starts_with([' ', '\t']));
		if let Some((name, _)) = nested.filter(|_| in_dependencies).and_then(|rest| rest.split_once(':')) {
			names.insert(name.trim().to_owned());
		}
	}
	names
}

struct Ruby;

const RUBY_DATABASES: [(&str, &str); 4] = [
	("activerecord", "activerecord"),
	("sequel", "sequel"),
	("rom-sql", "rom"),
	("mongoid", "mongoid"),
];

const RUBY_FRAMEWORKS: [(&str, &str); 5] = [
	("rails", "rails"),
	("sinatra", "sinatra"),
	("hanami", "hanami"),
	("roda", "roda"),
	("grape", "grape"),
];

impl Detector for Ruby {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let Some(text) = read_text(workspace, "Gemfile")? else {
			return Ok(());
		};
		stack.insert(Fact::Language, "ruby");
		stack.insert(Fact::PackageManager, "bundler");
		let names = gem_names(&text);
		insert_named(stack, Fact::Framework, &names, &RUBY_FRAMEWORKS);
		insert_named(stack, Fact::Database, &names, &RUBY_DATABASES);
		if names.contains("rails") {
			stack.insert(Fact::Database, "activerecord");
		}
		Ok(())
	}
}

/// Gem names in a `Gemfile`'s `gem "name"` lines.
fn gem_names(text: &str) -> BTreeSet<String> {
	text
		.lines()
		.filter_map(|line| {
			let rest = line.trim().strip_prefix("gem")?.trim_start_matches(['(', ' ', '\t']);
			let quote = rest.chars().next().filter(|c| matches!(c, '"' | '\''))?;
			let name = rest[1..].split(quote).next()?;
			(!name.is_empty()).then(|| name.to_owned())
		})
		.collect()
}

struct Php;

const PHP_FRAMEWORKS: [(&str, &str); 7] = [
	("laravel/framework", "laravel"),
	("symfony/framework-bundle", "symfony"),
	("slim/slim", "slim"),
	("cakephp/cakephp", "cakephp"),
	("codeigniter4/framework", "codeigniter"),
	("yiisoft/yii2", "yii"),
	("laminas/laminas-mvc", "laminas"),
];

const PHP_DATABASES: [(&str, &str); 2] = [("doctrine/orm", "doctrine"), ("illuminate/database", "eloquent")];

impl Detector for Php {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let Some(text) = read_text(workspace, "composer.json")? else {
			return Ok(());
		};
		stack.insert(Fact::Language, "php");
		stack.insert(Fact::PackageManager, "composer");
		let manifest = serde_json::from_str::<Json>(&text).unwrap_or_default();
		let names: BTreeSet<String> = json_keys(&manifest, &["require", "require-dev"]).collect();
		insert_named(stack, Fact::Framework, &names, &PHP_FRAMEWORKS);
		insert_named(stack, Fact::Database, &names, &PHP_DATABASES);
		if names.contains("laravel/framework") {
			stack.insert(Fact::Database, "eloquent");
		}
		Ok(())
	}
}

struct Jvm;

const GRADLE_FILES: [&str; 5] = [
	"build.gradle.kts",
	"build.gradle",
	"gradle/libs.versions.toml",
	"app/build.gradle.kts",
	"app/build.gradle",
];

const JVM_FRAMEWORKS: [(&str, &str); 16] = [
	("org.springframework.boot", "spring-boot"),
	("io.quarkus", "quarkus"),
	("io.micronaut", "micronaut"),
	("io.helidon", "helidon"),
	("io.vertx", "vertx"),
	("io.javalin", "javalin"),
	("io.dropwizard", "dropwizard"),
	("io.ktor", "ktor"),
	("com.android.application", "android"),
	("com.android.library", "android"),
	("com.android.tools.build", "android"),
	("org.jetbrains.compose", "compose-multiplatform"),
	("org.jetbrains.compose", "compose"),
	("androidx.compose", "compose"),
	("kotlin(\"multiplatform\")", "kotlin-multiplatform"),
	("org.jetbrains.kotlin.multiplatform", "kotlin-multiplatform"),
];

const JVM_DATABASES: [(&str, &str); 8] = [
	("org.hibernate", "hibernate"),
	("org.jetbrains.exposed", "exposed"),
	("org.jooq", "jooq"),
	("org.mybatis", "mybatis"),
	("spring-boot-starter-data-jpa", "spring-data-jpa"),
	("spring-data-jpa", "spring-data-jpa"),
	("androidx.room", "room"),
	("app.cash.sqldelight", "sqldelight"),
];

impl Detector for Jvm {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let mut gradle = Vec::new();
		for path in GRADLE_FILES {
			gradle.extend(read_text(workspace, path)?);
		}
		let maven = read_text(workspace, "pom.xml")?;
		let manager = match (gradle.is_empty(), &maven) {
			(false, _) => "gradle",
			(true, Some(_)) => "maven",
			(true, None) => return Ok(()),
		};
		stack.insert(Fact::PackageManager, manager);
		let text = gradle.into_iter().chain(maven).collect::<Vec<_>>().join("\n");
		let kotlin = ["kotlin(", "org.jetbrains.kotlin", "kotlin-stdlib"]
			.iter()
			.any(|needle| text.contains(needle));
		stack.insert(Fact::Language, if kotlin { "kotlin" } else { "java" });
		insert_contained(stack, Fact::Framework, &text, &JVM_FRAMEWORKS);
		insert_contained(stack, Fact::Database, &text, &JVM_DATABASES);
		Ok(())
	}
}

struct Swift;

const SWIFT_FRAMEWORKS: [(&str, &str); 2] = [
	("vapor/vapor", "vapor"),
	("hummingbird-project/hummingbird", "hummingbird"),
];

const SWIFT_DATABASES: [(&str, &str); 3] = [
	("vapor/fluent", "fluent"),
	("groue/GRDB.swift", "grdb"),
	("realm/realm-swift", "realm"),
];

impl Detector for Swift {
	fn detect(&self, workspace: &Workspace, stack: &mut Stack) -> Result<(), WorkspaceError> {
		let package = read_text(workspace, "Package.swift")?;
		let cocoapods = exists(workspace, "Podfile")?;
		let xcode = workspace
			.list(None)?
			.iter()
			.any(|name| name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace"));
		if package.is_none() && !cocoapods && !xcode {
			return Ok(());
		}
		stack.insert(Fact::Language, "swift");
		if cocoapods {
			stack.insert(Fact::PackageManager, "cocoapods");
		}
		if let Some(text) = package {
			stack.insert(Fact::PackageManager, "swiftpm");
			insert_contained(stack, Fact::Framework, &text, &SWIFT_FRAMEWORKS);
			insert_contained(stack, Fact::Database, &text, &SWIFT_DATABASES);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;
	use crate::workspace::MemoryFs;

	fn workspace(files: &[(&str, &str)]) -> Workspace {
		let fs = MemoryFs::new();
		for (name, text) in files {
			fs.insert(&RelPath::new(name).unwrap(), text.as_bytes());
		}
		Workspace::new(Arc::new(fs))
	}

	fn detect_in(files: &[(&str, &str)]) -> ProjectModel {
		detect(&workspace(files), &ProjectConfig::default()).unwrap()
	}

	fn has(model: &ProjectModel, fact: Fact, values: &[&str]) -> bool {
		values.iter().all(|value| model.stack.has(fact, value))
	}

	#[test]
	fn js_should_detect_frameworks_styling_typescript_and_bun() {
		let model = detect_in(&[
			(
				"package.json",
				r#"{"dependencies": {"next": "1", "react": "1", "@stylexjs/stylex": "0.18", "drizzle-orm": "1"}, "devDependencies": {"tailwindcss": "4", "typescript": "6"}}"#,
			),
			("bun.lock", ""),
		]);

		assert!(has(&model, Fact::Framework, &["next", "react"]));
		assert!(has(&model, Fact::Styling, &["tailwind", "stylex"]));
		assert!(has(&model, Fact::Database, &["drizzle"]));
		assert!(has(&model, Fact::Language, &["typescript"]));
		assert!(has(&model, Fact::Runtime, &["bun"]) && !model.stack.has(Fact::Runtime, "node"));
		assert!(has(&model, Fact::PackageManager, &["bun"]));
	}

	#[test]
	fn js_should_default_to_node_runtime_and_javascript() {
		let model = detect_in(&[
			("package.json", r#"{"dependencies": {"express": "5"}}"#),
			("package-lock.json", ""),
		]);

		assert!(has(&model, Fact::Runtime, &["node"]));
		assert!(has(&model, Fact::Language, &["javascript"]));
		assert!(has(&model, Fact::PackageManager, &["npm"]));
	}

	#[test]
	fn js_should_prefer_declared_package_manager() {
		let model = detect_in(&[
			("package.json", r#"{"packageManager": "pnpm@10.0.0"}"#),
			("package-lock.json", ""),
		]);

		assert!(model.stack.has(Fact::PackageManager, "pnpm"));
	}

	#[test]
	fn deno_should_read_jsonc_imports_without_package_json() {
		let config = r#"{
			// Fresh 2 app
			"imports": {
				"fresh": "jsr:@fresh/core@^2.0.0",
				"hono": "jsr:@hono/hono@^4", /* router */
				"preact": "npm:preact@10.26.0",
			},
		}"#;
		let model = detect_in(&[("deno.jsonc", config)]);

		assert!(has(&model, Fact::Framework, &["fresh", "hono", "preact"]));
		assert!(has(&model, Fact::Runtime, &["deno"]));
		assert!(has(&model, Fact::PackageManager, &["deno"]));
		assert!(has(&model, Fact::Language, &["typescript"]));
	}

	#[test]
	fn specifier_package_should_handle_scopes_and_versions() {
		assert_eq!(specifier_package("jsr:@std/path@^1/join").as_deref(), Some("@std/path"));
		assert_eq!(specifier_package("npm:react@19").as_deref(), Some("react"));
		assert_eq!(specifier_package("https://deno.land/x/oak/mod.ts"), None);
	}

	#[test]
	fn rust_should_detect_frameworks_runtime_and_databases() {
		let model = detect_in(&[(
			"Cargo.toml",
			"[dependencies]\nbevy = \"0.17\"\ntokio = { version = \"1\", features = [\"full\"] }\norm = { package = \"sea-orm\", version = \"1\" }\nsurrealdb = \"2\"\n\n[workspace.dependencies]\ndiesel = \"2\"\n",
		)]);

		assert!(has(&model, Fact::Framework, &["bevy"]));
		assert!(has(&model, Fact::Runtime, &["tokio"]));
		assert!(has(&model, Fact::Database, &["seaorm", "surrealdb", "diesel"]));
	}

	#[test]
	fn rust_gpui_kit_should_imply_gpui() {
		let model = detect_in(&[("Cargo.toml", "[dependencies]\ngpui-kit = \"0.6\"\n")]);

		assert!(has(&model, Fact::Framework, &["gpui-kit", "gpui"]));
	}

	#[test]
	fn python_should_read_pep621_and_normalize_names() {
		let pyproject = "[project]\ndependencies = [\"FastAPI[standard]>=0.115\", \"SQLAlchemy>=2\"]\n\n[dependency-groups]\ndev = [\"pytest\"]\n";
		let model = detect_in(&[("pyproject.toml", pyproject), ("uv.lock", "")]);

		assert!(has(&model, Fact::Framework, &["fastapi"]));
		assert!(has(&model, Fact::Database, &["sqlalchemy"]));
		assert!(has(&model, Fact::PackageManager, &["uv"]));
	}

	#[test]
	fn python_should_read_poetry_and_requirements() {
		let poetry = detect_in(&[(
			"pyproject.toml",
			"[tool.poetry.dependencies]\npython = \"^3.13\"\nDjango = \"^5\"\n",
		)]);
		let pip = detect_in(&[("requirements.txt", "# web\nlitestar==2.*\n-r base.txt\nflask-cors\n")]);

		assert!(has(&poetry, Fact::Framework, &["django"]) && has(&poetry, Fact::PackageManager, &["poetry"]));
		assert!(has(&pip, Fact::Framework, &["litestar"]) && !pip.stack.has(Fact::Framework, "flask"));
	}

	#[test]
	fn go_should_read_require_lines_and_blocks() {
		let go_mod = "module x\n\nrequire github.com/labstack/echo/v4 v4.13.0\n\nrequire (\n\tgorm.io/gorm v1.31.0 // indirect\n\tgithub.com/gin-gonic/gin v1.11.0\n)\n";
		let model = detect_in(&[("go.mod", go_mod)]);

		assert!(has(&model, Fact::Framework, &["echo", "gin"]));
		assert!(has(&model, Fact::Database, &["gorm"]));
		assert!(!model.stack.has(Fact::Framework, "fiber"));
	}

	#[test]
	fn dart_should_detect_flutter() {
		let model = detect_in(&[("pubspec.yaml", "dependencies:\n  flutter:\n    sdk: flutter\n")]);

		assert!(has(&model, Fact::Framework, &["flutter"]) && has(&model, Fact::Language, &["dart"]));
	}

	#[test]
	fn ruby_should_read_gem_lines() {
		let model = detect_in(&[(
			"Gemfile",
			"source \"https://rubygems.org\"\ngem \"rails\", \"~> 8.1\"\ngem 'sinatra'\n",
		)]);

		assert!(has(&model, Fact::Framework, &["rails", "sinatra"]));
	}

	#[test]
	fn php_should_read_composer_require() {
		let model = detect_in(&[(
			"composer.json",
			r#"{"require": {"laravel/framework": "^12"}, "require-dev": {"doctrine/orm": "^3"}}"#,
		)]);

		assert!(has(&model, Fact::Framework, &["laravel"]) && has(&model, Fact::Database, &["doctrine"]));
	}

	#[test]
	fn java_should_detect_spring_boot_with_maven() {
		let model = detect_in(&[(
			"pom.xml",
			"<parent><groupId>org.springframework.boot</groupId></parent><dependency><groupId>org.hibernate</groupId></dependency>",
		)]);

		assert!(has(&model, Fact::Language, &["java"]) && has(&model, Fact::PackageManager, &["maven"]));
		assert!(has(&model, Fact::Framework, &["spring-boot"]) && has(&model, Fact::Database, &["hibernate"]));
	}

	#[test]
	fn kotlin_should_detect_jetpack_compose() {
		let model = detect_in(&[(
			"app/build.gradle.kts",
			"plugins { id(\"com.android.application\") }\ndependencies { implementation(platform(\"androidx.compose:compose-bom:2026.10.00\")) }\n",
		)]);

		assert!(has(&model, Fact::Framework, &["android", "compose"]));
	}

	#[test]
	fn kotlin_should_detect_ktor_and_android_from_version_catalog() {
		let model = detect_in(&[
			("build.gradle.kts", "plugins { kotlin(\"jvm\") }\n"),
			(
				"gradle/libs.versions.toml",
				"[libraries]\nktor = { module = \"io.ktor:ktor-server-core\" }\n[plugins]\nandroid-application = { id = \"com.android.application\" }\n",
			),
		]);

		assert!(has(&model, Fact::Language, &["kotlin"]) && has(&model, Fact::PackageManager, &["gradle"]));
		assert!(has(&model, Fact::Framework, &["ktor", "android"]));
	}

	#[test]
	fn swift_should_detect_vapor_with_swiftpm() {
		let model = detect_in(&[(
			"Package.swift",
			".package(url: \"https://github.com/vapor/vapor.git\", from: \"4.0.0\"),\n.package(url: \"https://github.com/vapor/fluent.git\", from: \"4.0.0\")",
		)]);

		assert!(has(&model, Fact::Language, &["swift"]) && has(&model, Fact::PackageManager, &["swiftpm"]));
		assert!(has(&model, Fact::Framework, &["vapor"]) && has(&model, Fact::Database, &["fluent"]));
	}

	#[test]
	fn swift_should_detect_xcode_projects() {
		let fs = MemoryFs::new();
		fs.insert(&RelPath::new("App.xcodeproj/project.pbxproj").unwrap(), b"".to_vec());
		let model = detect(&Workspace::new(Arc::new(fs)), &ProjectConfig::default()).unwrap();

		assert!(has(&model, Fact::Language, &["swift"]));
	}

	#[test]
	fn database_should_ignore_drivers_and_clients() {
		let js = detect_in(&[("package.json", r#"{"dependencies": {"pg": "8", "ioredis": "5"}}"#)]);
		let python = detect_in(&[("requirements.txt", "psycopg2-binary==2.9\n")]);
		let go = detect_in(&[("go.mod", "module x\nrequire github.com/jackc/pgx/v5 v5.7.0\n")]);

		for model in [js, python, go] {
			assert!(model.stack.values(Fact::Database).next().is_none());
		}
	}

	#[test]
	fn dart_should_detect_database_from_pubspec() {
		let model = detect_in(&[(
			"pubspec.yaml",
			"dependencies:\n  drift: ^2.0.0\n  sqflite: ^2.0.0\n  flutter:\n    sdk: flutter\n",
		)]);

		assert!(has(&model, Fact::Database, &["drift"]) && has(&model, Fact::Framework, &["flutter"]));
		assert!(!has(&model, Fact::Database, &["sqflite"]));
	}

	#[test]
	fn database_should_include_orms_built_into_frameworks() {
		let rails = detect_in(&[("Gemfile", "gem \"rails\"\n")]);
		let laravel = detect_in(&[("composer.json", r#"{"require": {"laravel/framework": "^12"}}"#)]);
		let django = detect_in(&[("requirements.txt", "django>=5\n")]);

		assert!(has(&rails, Fact::Database, &["activerecord"]));
		assert!(has(&laravel, Fact::Database, &["eloquent"]));
		assert!(has(&django, Fact::Database, &["django-orm"]));
	}

	#[test]
	fn database_should_ignore_sql_and_schema_files() {
		let model = detect_in(&[
			("package.json", "{}"),
			("schema.sql", "create table users (id int);"),
			("db/migrations/001_init.sql", "create table posts (id int);"),
			("prisma/schema.prisma", "datasource db { provider = \"postgresql\" }"),
		]);

		assert!(model.stack.values(Fact::Database).next().is_none());
	}

	#[test]
	fn elixir_and_dotnet_should_not_be_detected() {
		let model = detect_in(&[
			("mix.exs", "{:phoenix, \"~> 1.7\"}"),
			("App.csproj", "<Project Sdk=\"Microsoft.NET.Sdk.Web\"/>"),
		]);

		assert!(model.stack.is_empty());
	}

	#[test]
	fn env_should_read_names_but_not_values() {
		let model = detect_in(&[(".env.example", "# comment\nAPI_URL=https://x\nexport SECRET=\n")]);

		assert_eq!(model.env.into_iter().collect::<Vec<_>>(), ["API_URL", "SECRET"]);
	}

	#[test]
	fn config_should_override_detection() {
		let workspace = workspace(&[("package.json", r#"{"dependencies": {"react": "1"}}"#)]);
		let mut config = ProjectConfig::default();
		config.stack.insert(Fact::Framework, vec!["svelte".to_owned()]);

		let model = detect(&workspace, &config).unwrap();

		assert!(model.stack.has(Fact::Framework, "svelte") && !model.stack.has(Fact::Framework, "react"));
	}
}
