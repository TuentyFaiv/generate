use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::config::CLIConfig;
use crate::cli::structs::Answers;

pub struct CLIReactTemplates {
  answers: Answers,
  config: CLIConfig,
}

impl CLIReactTemplates {
  pub fn new(config: CLIConfig, answers: Answers) -> Self {
    Self { config, answers }
  }
  pub fn generate_layout(&self) -> Result<()> {
    println!("generate_layout");
    Ok(())
  }
  pub fn generate_context(&self) -> Result<()> {
    println!("generate_context");
    Ok(())
  }
  pub fn generate_hoc(&self) -> Result<()> {
    println!("generate_hoc");
    Ok(())
  }
  pub fn generate_hook(&self) -> Result<()> {
    println!("generate_hook");
    Ok(())
  }
}