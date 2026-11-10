//! Repository automation: `cargo xtask <task>`, running the same checks as CI.

use std::env;
use std::fmt;
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  fmt     Check Rust formatting
  lint    Run clippy with warnings denied
  doc     Build the docs with warnings denied
  test    Run the workspace tests
  deny    Check dependencies with cargo-deny (requires cargo-deny)
  js      Check JS/JSON formatting and lints with Biome (requires bun)
  ci      Run fmt, lint, doc, test, deny and js in order
  schema  Regenerate the JSON Schemas in schemas/";

/// Repository task made of one or more commands.
#[derive(Clone, Copy, Debug)]
enum Task {
	Fmt,
	Lint,
	Doc,
	Test,
	Deny,
	Js,
	Ci,
	Schema,
}

impl Task {
	fn parse(name: &str) -> Option<Self> {
		match name {
			"fmt" => Some(Self::Fmt),
			"lint" => Some(Self::Lint),
			"doc" => Some(Self::Doc),
			"test" => Some(Self::Test),
			"deny" => Some(Self::Deny),
			"js" => Some(Self::Js),
			"ci" => Some(Self::Ci),
			"schema" => Some(Self::Schema),
			_ => None,
		}
	}

	fn steps(self) -> Vec<Step> {
		match self {
			Self::Fmt => vec![Step::cargo(&["fmt", "--all", "--check"])],
			Self::Lint => vec![
				Step::cargo(&[
					"clippy",
					"--workspace",
					"--all-targets",
					"--all-features",
					"--locked",
					"--",
					"-D",
					"warnings",
				]),
				Step::cargo(&[
					"clippy",
					"--package",
					"tfverse-packs",
					"--all-targets",
					"--no-default-features",
					"--locked",
					"--",
					"-D",
					"warnings",
				]),
			],
			Self::Doc => vec![
				Step::cargo(&["doc", "--workspace", "--all-features", "--no-deps", "--locked"])
					.env("RUSTDOCFLAGS", "-D warnings"),
			],
			Self::Test => vec![Step::cargo(&["test", "--workspace", "--all-features", "--locked"])],
			Self::Deny => vec![Step::cargo(&["deny", "--all-features", "check"])],
			Self::Js => vec![Step::new("bunx", &["biome", "ci", "."])],
			Self::Schema => vec![
				Step::cargo(&["test", "--package", "tfverse-core", "--lib", "--locked", "schema::"])
					.env("TFVERSE_UPDATE_SCHEMAS", "1"),
			],
			Self::Ci => [Self::Fmt, Self::Lint, Self::Doc, Self::Test, Self::Deny, Self::Js]
				.into_iter()
				.flat_map(Self::steps)
				.collect(),
		}
	}
}

/// One external command to run.
#[derive(Debug)]
struct Step {
	program: String,
	args: Vec<String>,
	envs: Vec<(&'static str, &'static str)>,
}

impl Step {
	fn new(program: &str, args: &[&str]) -> Self {
		Self {
			program: program.to_owned(),
			args: args.iter().map(|&arg| arg.to_owned()).collect(),
			envs: Vec::new(),
		}
	}

	fn cargo(args: &[&str]) -> Self {
		let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
		Self::new(&cargo, args)
	}

	fn env(mut self, key: &'static str, value: &'static str) -> Self {
		self.envs.push((key, value));
		self
	}

	fn run(&self) -> Result<(), StepError> {
		let status = Command::new(&self.program)
			.args(&self.args)
			.envs(self.envs.iter().copied())
			.status()
			.map_err(|source| StepError::Spawn {
				step: self.to_string(),
				source,
			})?;

		if status.success() {
			Ok(())
		} else {
			Err(StepError::Failed { step: self.to_string() })
		}
	}
}

impl fmt::Display for Step {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (key, value) in &self.envs {
			write!(f, "{key}=\"{value}\" ")?;
		}
		write!(f, "{} {}", self.program, self.args.join(" "))
	}
}

/// Why a step did not succeed.
#[derive(Debug)]
enum StepError {
	Spawn { step: String, source: std::io::Error },
	Failed { step: String },
}

impl fmt::Display for StepError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Spawn { step, source } => write!(f, "could not start `{step}`: {source}"),
			Self::Failed { step } => write!(f, "`{step}` failed"),
		}
	}
}

fn main() -> ExitCode {
	let Some(name) = env::args().nth(1) else {
		eprintln!("{USAGE}");
		return ExitCode::FAILURE;
	};

	let Some(task) = Task::parse(&name) else {
		eprintln!("unknown task `{name}`\n\n{USAGE}");
		return ExitCode::FAILURE;
	};

	for step in task.steps() {
		eprintln!("→ {step}");
		if let Err(error) = step.run() {
			eprintln!("✗ {error}");
			return ExitCode::FAILURE;
		}
	}

	ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_should_reject_unknown_task() {
		assert!(Task::parse("deploy").is_none());
	}

	#[test]
	fn ci_should_run_every_check_in_order() {
		let steps: Vec<String> = Task::Ci.steps().iter().map(|step| step.args.join(" ")).collect();

		assert_eq!(
			steps,
			[
				"fmt --all --check",
				"clippy --workspace --all-targets --all-features --locked -- -D warnings",
				"clippy --package tfverse-packs --all-targets --no-default-features --locked -- -D warnings",
				"doc --workspace --all-features --no-deps --locked",
				"test --workspace --all-features --locked",
				"deny --all-features check",
				"biome ci .",
			]
		);
	}

	#[test]
	fn doc_should_deny_rustdoc_warnings() {
		let steps = Task::Doc.steps();

		assert_eq!(steps[0].envs, [("RUSTDOCFLAGS", "-D warnings")]);
	}
}
