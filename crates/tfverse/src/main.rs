//! `tfverse` command-line entry point.

use clap::{CommandFactory, Parser};
use tfverse_core::brand;

/// Generate projects and code from versioned, updatable packs, for humans and AI agents.
#[derive(Debug, Parser)]
#[command(name = brand::BINARY_NAME, version, about, long_about = None)]
struct Cli {}

fn main() -> anyhow::Result<()> {
	let Cli {} = Cli::parse();

	Cli::command().print_help()?;

	Ok(())
}
