use anyhow::Result;

use crate::config::CLIConfig;
use crate::cli::structs::Answers;

pub struct CLISvelteTemplates {
  answers: Answers,
  config: CLIConfig,
}

impl CLISvelteTemplates {
  pub fn new(config: CLIConfig, answers: Answers) -> Self {
    Self { config, answers }
  }
  pub fn generate_layout(&self) -> Result<()> {
    println!("generate_layout");
    Ok(())
  }
}