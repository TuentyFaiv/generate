//! Black-box tests for the `tfverse` binary.

use std::error::Error;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt as _;
use predicates::str::{contains, starts_with};
use tfverse_core::brand;

type TestResult = Result<(), Box<dyn Error>>;

fn tfverse() -> Result<Command, Box<dyn Error>> {
	Ok(Command::cargo_bin(brand::BINARY_NAME)?)
}

#[test]
fn version_flag_should_print_name_and_package_version() -> TestResult {
	let expected = format!("{} {}", brand::BINARY_NAME, env!("CARGO_PKG_VERSION"));

	tfverse()?
		.arg("--version")
		.assert()
		.success()
		.stdout(starts_with(expected));

	Ok(())
}

#[test]
fn no_arguments_should_print_usage() -> TestResult {
	tfverse()?.assert().success().stdout(contains("Usage:"));

	Ok(())
}

#[test]
fn unknown_argument_should_fail_with_usage_error() -> TestResult {
	tfverse()?.arg("--definitely-not-a-flag").assert().code(2);

	Ok(())
}

#[test]
fn rpc_should_answer_json_rpc_on_stdio() -> TestResult {
	let dir = tempfile::tempdir()?;
	let request = r#"{"jsonrpc":"2.0","id":1,"method":"history.list"}"#;

	tfverse()?
		.arg("rpc")
		.current_dir(dir.path())
		.write_stdin(format!("{request}\n"))
		.assert()
		.success()
		.stdout(contains(r#""id":1"#).and(contains(r#""result":[]"#)));
	Ok(())
}

#[test]
fn rpc_should_stay_out_of_help() -> TestResult {
	tfverse()?
		.arg("--help")
		.assert()
		.success()
		.stdout(contains("rpc").not());
	Ok(())
}

/// A React project with an isolated user config and pack cache.
fn react_project() -> Result<tempfile::TempDir, Box<dyn Error>> {
	let dir = tempfile::tempdir()?;
	std::fs::write(
		dir.path().join("package.json"),
		r#"{"dependencies": {"react": "19"}, "devDependencies": {"typescript": "6"}}"#,
	)?;
	std::fs::create_dir(dir.path().join(".home"))?;
	Ok(dir)
}

fn in_project(dir: &tempfile::TempDir) -> Result<Command, Box<dyn Error>> {
	let mut command = tfverse()?;
	command
		.current_dir(dir.path())
		.env("TFVERSE_CONFIG_DIR", dir.path().join(".home/config"))
		.env("TFVERSE_CACHE_DIR", dir.path().join(".home/cache"))
		.env("NO_COLOR", "1");
	Ok(command)
}

#[test]
fn add_should_plan_then_apply_only_with_yes() -> TestResult {
	let dir = react_project()?;

	in_project(&dir)?
		.args(["add", "ui/atom", "Button", "--dry-run"])
		.assert()
		.success()
		.stdout(contains("create  src/ui/atoms/button/button.tsx"));
	in_project(&dir)?
		.args(["add", "ui/atom", "Button"])
		.assert()
		.code(2)
		.stderr(contains("--yes"));
	assert!(!dir.path().join("src").exists());
	in_project(&dir)?
		.args(["add", "atom", "Button", "--yes"])
		.assert()
		.success()
		.stdout(contains("Applied transaction 1"));

	assert!(dir.path().join("src/ui/atoms/button/button.tsx").is_file());
	Ok(())
}

#[test]
fn json_output_should_describe_the_plan_and_the_transaction() -> TestResult {
	let dir = react_project()?;

	let output = in_project(&dir)?
		.args(["--json", "add", "ui/atom", "Chip", "--yes"])
		.output()?;
	let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;

	assert!(output.status.success());
	assert_eq!(value["plan"]["summary"], "add ui/atom");
	assert_eq!(value["applied"]["id"], 1);
	Ok(())
}

#[test]
fn status_why_and_undo_should_follow_the_edits() -> TestResult {
	let dir = react_project()?;
	in_project(&dir)?
		.args(["add", "ui/atom", "Badge", "--yes"])
		.assert()
		.success();
	let file = dir.path().join("src/ui/atoms/badge/badge.tsx");
	std::fs::write(&file, format!("{}// mine\n", std::fs::read_to_string(&file)?))?;
	let last = std::fs::read_to_string(&file)?.lines().count();

	in_project(&dir)?
		.arg("status")
		.assert()
		.success()
		.stdout(contains("modified  src/ui/atoms/badge/badge.tsx"));
	in_project(&dir)?
		.args(["why", &format!("src/ui/atoms/badge/badge.tsx:{last}")])
		.assert()
		.success()
		.stdout(contains("is yours"));
	in_project(&dir)?
		.arg("undo")
		.assert()
		.code(1)
		.stderr(contains("changed after transaction 1"));
	in_project(&dir)?.args(["undo", "--force"]).assert().success();

	assert!(!file.exists());
	Ok(())
}

#[test]
fn a_rejected_plan_should_exit_with_three() -> TestResult {
	let dir = react_project()?;
	std::fs::create_dir_all(dir.path().join("src/ui/atoms/button"))?;
	std::fs::write(dir.path().join("src/ui/atoms/button/button.tsx"), "mine\n")?;

	in_project(&dir)?
		.args(["add", "ui/atom", "Button", "--yes"])
		.assert()
		.code(3)
		.stderr(contains("already exists"));
	Ok(())
}

#[test]
fn capture_should_make_a_piece_usable_with_from() -> TestResult {
	let dir = react_project()?;
	std::fs::create_dir_all(dir.path().join("src/widgets"))?;
	std::fs::write(
		dir.path().join("src/widgets/price-tag.ts"),
		"export const PriceTag = 'price-tag'\n",
	)?;

	in_project(&dir)?
		.args([
			"pack",
			"capture",
			"src/widgets/price-tag.ts",
			"--piece",
			"widget",
			"--name",
			"PriceTag",
			"--yes",
		])
		.assert()
		.success();
	in_project(&dir)?
		.args(["add", "local/widget", "SaleBadge", "--from", "./packs/local", "--yes"])
		.assert()
		.success();

	assert_eq!(
		std::fs::read_to_string(dir.path().join("src/widgets/sale-badge.ts"))?,
		"export const SaleBadge = 'sale-badge'\n"
	);
	Ok(())
}

#[test]
fn pack_list_should_include_the_official_packs() -> TestResult {
	let dir = react_project()?;

	in_project(&dir)?
		.args(["pack", "list"])
		.assert()
		.success()
		.stdout(contains("ui 0.1.0").and(contains("backend 0.1.0")));
	Ok(())
}
