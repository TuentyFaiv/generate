mod statics;

// use anyhow::Result;

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
}