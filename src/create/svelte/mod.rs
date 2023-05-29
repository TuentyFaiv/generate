use anyhow::{Result, anyhow};

use crate::config::CLIConfig;
use crate::cli::structs::Answers;

pub struct CLISvelteCreation {
  answers: Answers,
  config: CLIConfig,
  error: String,
}

impl CLISvelteCreation {
  pub fn new(config: CLIConfig, answers: Answers, error: String) -> Self {
    Self { config, answers, error }
  }
  pub fn make_store(&self) -> Result<String> {
    println!("make_hoc");
    Err(anyhow!(self.error.clone()))
  }
  pub fn make_action(&self) -> Result<String> {
    println!("make_hook");
    Err(anyhow!(self.error.clone()))
  }
}
