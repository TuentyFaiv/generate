mod project;
mod page;
mod layout;
mod component;
mod schema;
mod service;
// mod context;

use anyhow::Result;

use crate::statics::NOT_IMPLEMENTED;
use crate::cli::structs::Answers;
use crate::config::CLIConfig;
use crate::create::structs::{
  ComponentCreation,
  PageCreation,
  LayoutCreation,
  SchemaCreation,
};

use super::utils;

pub struct CLIGlobalTemplates {
  answers: Answers,
  config: CLIConfig,
  error: String,
}

impl CLIGlobalTemplates {
  pub fn new(config: CLIConfig, answers: Answers) -> Self {
    let error = format!("{} Repository not found", NOT_IMPLEMENTED);
    Self { config, answers, error, }
  }
  pub fn generate_component(&self, templates: ComponentCreation) -> Result<()> {
    component::generate(&self, &templates)
  }
  pub fn generate_project(&self) -> Result<()> {
    project::generate(&self)
  }
  pub fn generate_page(&self, templates: PageCreation) -> Result<()> {
    page::generate(&self, &templates)
  }
  pub fn generate_layout(&self, templates: LayoutCreation) -> Result<()> {
    layout::generate(&self, &templates)
  }
  pub fn generate_schema(&self, templates: SchemaCreation) -> Result<()> {
    schema::generate(&self, &templates)
  }
  pub fn generate_service(&self) -> Result<()> {
    service::generate(&self)
  }
}