// pub mod component;
// pub mod project;
// pub mod page;
// pub mod layout;
// pub mod schema;
// pub mod service;

use anyhow::Result;

use crate::config::Config;
use crate::cli::questions::Answers;

pub struct CLIGlobalCreation<'a, C: Config> {
  answers: Answers,
  config: &'a C,
}

impl <'a, C: Config> CLIGlobalCreation<'a, C> {
  pub fn new(config: &'a C, answers: Answers) -> Self {
    Self { config, answers }
  }
  pub fn make_component(&self) -> Result<()> {
    println!("make_component");
    Ok(())
  }
  pub fn make_project(&self) -> Result<()> {
    println!("make_project");
    Ok(())
  }
  pub fn make_page(&self) -> Result<()> {
    println!("make_page");
    Ok(())
  }
  pub fn make_layout(&self) -> Result<()> {
    println!("make_layout");
    Ok(())
  }
  pub fn make_schema(&self) -> Result<()> {
    println!("make_schema");
    Ok(())
  }
  pub fn make_service(&self) -> Result<()> {
    println!("make_service");
    Ok(())
  }
}