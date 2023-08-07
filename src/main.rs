mod utils;
mod cli;
mod statics;
mod templates;
mod create;
mod config;

use anyhow::Result;
use clap::Parser;

use config::CLIConfig;
use cli::structs::Args;
use cli::{Questions, CLIQuestions};
use cli::utils::msg;
use create::CLICreation;


pub fn main() -> Result<()> {
	let args = Args::parse();

	let cli_config = CLIConfig::new(args.clone());

	let questions = CLIQuestions::new(cli_config.clone(), args);
	let answers = questions.ask()?;

	// println!("{:?}", answers.clone());
	
	if !answers.accept { return Ok(()); }

	msg(&CLICreation::new(cli_config, answers).create()?);

	Ok(())
}
