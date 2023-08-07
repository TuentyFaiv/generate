// use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
// use console::style;

// use crate::cli::{utils::done, enums::Tool, structs::Answers};
// use crate::statics;
// use crate::statics::OK;

use super::CLIGlobalCreation;

pub fn create(CLIGlobalCreation {
  // answers,
  // config,
  error,
  // global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  println!("make_context");
  Err(anyhow!(error.clone()))
}