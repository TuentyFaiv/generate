#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ArchType {
  Project,
  Library,
  Component,
  Hoc,
  Hook,
  Context,
  Layout,
  Page,
  Service,
  Schema,
  Action,
  Store,
  Class,
}

impl ArchType {
  pub fn parse(name: &str) -> ArchType {
    match name {
      "project" => ArchType::Project,
      "library" => ArchType::Library,
      "component" => ArchType::Component,
      "hoc" => ArchType::Hoc,
      "hook" => ArchType::Hook,
      "context" => ArchType::Context,
      "layout" => ArchType::Layout,
      "page" => ArchType::Page,
      "service" => ArchType::Service,
      "schema" => ArchType::Schema,
      "action" => ArchType::Action,
      "store" => ArchType::Store,
      "class" => ArchType::Class,
      _ => ArchType::Project,
    }
  }
  pub fn to_string(&self) -> String {
    match self {
      ArchType::Project => "project".to_string(),
      ArchType::Library => "library".to_string(),
      ArchType::Component => "component".to_string(),
      ArchType::Hoc => "hoc".to_string(),
      ArchType::Hook => "hook".to_string(),
      ArchType::Context => "context".to_string(),
      ArchType::Layout => "layout".to_string(),
      ArchType::Page => "page".to_string(),
      ArchType::Service => "service".to_string(),
      ArchType::Schema => "schema".to_string(),
      ArchType::Action => "action".to_string(),
      ArchType::Store => "store".to_string(),
      ArchType::Class => "class".to_string(),
    }
  }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Tool {
  React,
  Svelte,
  Vanilla,
}

impl Tool {
  pub fn parse(name: &str) -> Tool {
    match name {
      "react" => Tool::React,
      "svelte" => Tool::Svelte,
      "vanilla" => Tool::Vanilla,
      _ => Tool::React,
    }
  }
  pub fn to_string(&self) -> String {
    match self {
      Tool::React => "react".to_string(),
      Tool::Svelte => "svelte".to_string(),
      Tool::Vanilla => "vanilla".to_string(),
    }
  }
}