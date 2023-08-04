mod project;
mod page;
mod layout;
mod component;
mod schema;
mod service;
mod context;

use anyhow::Result;
use super::structs;

use crate::config::CLIConfig;
use crate::cli::structs::Answers;
use crate::templates::global::CLIGlobalTemplates;
use crate::templates::react::CLIReactTemplates;
use crate::templates::svelte::CLISvelteTemplates;

pub struct CLIGlobalCreation {
  answers: Answers,
  config: CLIConfig,
  global: CLIGlobalTemplates,
  react: CLIReactTemplates,
  svelte: CLISvelteTemplates,
  error: String,
}

impl CLIGlobalCreation {
  pub fn new(config: CLIConfig, answers: Answers, error: String) -> Self {
    let global = CLIGlobalTemplates::new(config.clone(), answers.clone());
    let react = CLIReactTemplates::new(config.clone(), answers.clone());
    let svelte = CLISvelteTemplates::new(config.clone(), answers.clone());

    Self { config, answers, global, react, svelte, error }
  }
  pub fn make_component(&self) -> Result<String> {
    component::create(&self)
  }
  pub fn make_project(&self) -> Result<String> {
    project::create(&self)
  }
  pub fn make_page(&self) -> Result<String> {
    page::create(&self)
  }
  pub fn make_layout(&self) -> Result<String> {
    layout::create(&self)
  }
  pub fn make_schema(&self) -> Result<String> {
    schema::create(&self)
  }
  pub fn make_service(&self) -> Result<String> {
    service::create(&self)
  }
  pub fn make_context(&self) -> Result<String> {
    context::create(&self)
  }
}