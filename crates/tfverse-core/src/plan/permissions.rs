//! Write permissions, paths no pack may touch, and paths every plan flags as risky.

use crate::brand;
use crate::model::{Glob, Permissions, RelPath};

const FORBIDDEN: [&str; 3] = [".git/**", concat!(".", crate::brand_name!(), "/**"), brand::LOCK_FILE];

const RISKY: [(&str, &str); 22] = [
	("**/package.json", "package scripts run on install"),
	("**/.npmrc", "changes how packages install"),
	("**/*.config.js", "build config runs as code"),
	("**/*.config.ts", "build config runs as code"),
	("**/*.config.mjs", "build config runs as code"),
	("**/*.config.cjs", "build config runs as code"),
	("**/build.rs", "Cargo build scripts run as code"),
	("**/Cargo.toml", "changes Rust dependencies and build"),
	("**/pyproject.toml", "changes Python dependencies and build"),
	("**/Makefile", "build recipes run as code"),
	("**/Dockerfile", "container build runs as code"),
	(".github/**", "CI workflows run with repository secrets"),
	(".gitlab-ci.yml", "CI pipelines run with repository secrets"),
	(".husky/**", "git hooks run on commit"),
	(".vscode/**", "editor tasks and settings can run code"),
	(".idea/**", "editor settings can run code"),
	(".claude/**", "AI agent configuration"),
	(".cursor/**", "AI agent configuration"),
	(".codex/**", "AI agent configuration"),
	("**/AGENTS.md", "AI agent instructions"),
	("**/CLAUDE.md", "AI agent instructions"),
	("**/.env*", "may hold secrets"),
];

/// Whether no pack may ever write `path`.
pub(crate) fn is_forbidden(path: &RelPath) -> bool {
	FORBIDDEN.iter().any(|pattern| matches(pattern, path))
}

/// Why writing `path` deserves the user's attention, if it does.
pub(crate) fn risk(path: &RelPath) -> Option<&'static str> {
	RISKY
		.iter()
		.find(|(pattern, _)| matches(pattern, path))
		.map(|(_, reason)| *reason)
}

fn matches(pattern: &str, path: &RelPath) -> bool {
	Glob::new(pattern).is_ok_and(|glob| glob.matches(path))
}

impl Permissions {
	/// Whether these permissions allow writing `path`.
	#[must_use]
	pub fn allows_write(&self, path: &RelPath) -> bool {
		self.write.iter().any(|glob| glob.matches(path))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn rel(path: &str) -> RelPath {
		RelPath::new(path).unwrap()
	}

	#[test]
	fn git_internals_should_be_forbidden() {
		assert!(is_forbidden(&rel(".git/hooks/pre-commit")));
	}

	#[test]
	fn state_dir_and_lockfile_should_be_forbidden() {
		assert!(is_forbidden(&rel(".tfverse/objects/x")) && is_forbidden(&rel("tfverse.lock")));
	}

	#[test]
	fn nested_package_json_should_be_risky() {
		assert!(risk(&rel("apps/web/package.json")).is_some());
	}

	#[test]
	fn plain_source_file_should_not_be_risky() {
		assert_eq!(risk(&rel("src/components/Card.tsx")), None);
	}

	#[test]
	fn allows_write_should_follow_globs() {
		let permissions = Permissions {
			write: vec![Glob::new("src/**").unwrap()],
			..Permissions::default()
		};

		assert!(permissions.allows_write(&rel("src/a.ts")) && !permissions.allows_write(&rel("lib/a.ts")));
	}
}
