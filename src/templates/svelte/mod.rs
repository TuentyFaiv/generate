use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::config::CLIConfig;
use crate::cli::structs::Answers;
use crate::utils::read_path;

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