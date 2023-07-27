mod statics;

use anyhow::Result;

use crate::config::CLIConfig;
use crate::cli::structs::Answers;

pub struct CLIVanillaTemplates {
  answers: Answers,
  config: CLIConfig,
}

impl CLIVanillaTemplates {
  pub fn new(config: CLIConfig, answers: Answers) -> Self {
    Self { config, answers }
  }
  pub fn generate_page(&self) -> Result<()> {
    println!("generate_page");
    Ok(())
  }
  pub fn generate_layout(&self) -> Result<()> {
    println!("generate_layout");
    Ok(())
  }
}