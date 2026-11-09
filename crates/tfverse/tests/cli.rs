//! Black-box tests for the `tfverse` binary.

use std::error::Error;

use assert_cmd::Command;
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
