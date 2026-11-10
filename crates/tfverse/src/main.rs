//! `tfverse` command-line entry point.

mod cli;
mod commands;
mod context;
mod inputs;
mod ui;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use serde_json::json;
use tfverse_core::EngineError;

use crate::cli::{Cli, Command};
use crate::commands::Session;
use crate::context::Context;
use crate::ui::Output;

/// Exit codes beyond 0 (done) and 1 (failed).
pub(crate) const USAGE: u8 = 2;
const REJECTED: u8 = 3;

/// A failure that has already been reported, ending with `code`.
#[derive(Debug, thiserror::Error)]
#[error("exit {0}")]
pub(crate) struct Exit(pub(crate) u8);

fn main() -> ExitCode {
	let cli = Cli::parse();
	match run(&cli) {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			if let Some(Exit(code)) = error.downcast_ref::<Exit>() {
				return ExitCode::from(*code);
			}
			if let Some(EngineError::Rejected(rejected)) = error.downcast_ref::<EngineError>() {
				if cli.json {
					Output::json(&json!({ "error": "plan rejected", "problems": rejected.problems }));
				} else {
					eprint!("{}", Output::new(false, false).rejected(rejected));
				}
				return ExitCode::from(REJECTED);
			}
			if cli.json {
				Output::json(&json!({ "error": format!("{error:#}") }));
			} else {
				eprintln!("error: {error:#}");
			}
			ExitCode::FAILURE
		}
	}
}

fn run(cli: &Cli) -> anyhow::Result<()> {
	let root = match &cli.root {
		Some(root) => root.clone(),
		None => std::env::current_dir()?,
	};
	let interactive = !cli.json && !cli.no_input && io::stdin().is_terminal() && io::stderr().is_terminal();
	let Some(command) = &cli.command else {
		Cli::command().print_help()?;
		return Ok(());
	};
	match command {
		Command::Mcp => {
			let user = tfverse_core::UserConfig::load()?;
			tfverse_mcp::serve(&root, user, io::stdin().lock(), io::stdout())?;
			Ok(())
		}
		Command::Rpc => {
			tfverse_rpc::serve(&root, io::stdin().lock(), io::stdout())?;
			Ok(())
		}
		command => {
			let mut context = Context::open(&root)?;
			let session = Session {
				json: cli.json,
				interactive,
			};
			session.run(&mut context, command)
		}
	}
}
