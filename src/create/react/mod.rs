// pub mod context;
// pub mod hoc;
// pub mod hook;

use anyhow::Result;

use crate::config::Config;
use crate::cli::questions::{Answers};

pub struct CLIReactCreation<'a, C: Config> {
  answers: Answers,
  config: &'a C,
}

impl <'a, C: Config> CLIReactCreation<'a, C> {
  pub fn new(config: &'a C, answers: Answers) -> Self {
    Self { config, answers }
  }
  pub fn make_hoc(&self) -> Result<()> {
    println!("make_hoc");
    Ok(())
  }
  pub fn make_hook(&self) -> Result<()> {
    println!("make_hook");
    Ok(())
  }
  pub fn make_context(&self) -> Result<()> {
    println!("make_context");
    Ok(())
  }
}
