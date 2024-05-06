use anyhow::Result;
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::statics::OK;

use super::CLIGlobalCreation;

pub fn create(CLIGlobalCreation {
  answers,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { path, .. } = &answers;

  global.generate_project()?;

  done();
  Ok(format!("{} {}", OK, style(format!("Move to {} and start a new universe", path)).cyan()))
}