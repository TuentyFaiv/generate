// use anyhow::Result;

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
}