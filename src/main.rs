mod utils;
mod cli;
mod statics;
mod templates;
mod create;
mod config;

use anyhow::{Result};
use clap::Parser;

use config::CLIConfig;
use cli::questions::Questions;
use cli::Args;
use cli::questions::CLIQuestions;
use templates::CLITemplates;

pub fn main() -> Result<()> {
	let args = Args::parse();

	let cli_config = CLIConfig::new();

	let questions = CLIQuestions::new(&cli_config, args);

	let answers = questions.ask()?;
	
	if !answers.accept { return Ok(()); }

	let cli_templates = CLITemplates::new(&cli_config, answers);

	cli_templates.create()?;

	Ok(())
}
