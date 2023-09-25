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
      ArchType::Project => "project".to_owned(),
      ArchType::Library => "library".to_owned(),
      ArchType::Component => "component".to_owned(),
      ArchType::Hoc => "hoc".to_owned(),
      ArchType::Hook => "hook".to_owned(),
      ArchType::Context => "context".to_owned(),
      ArchType::Layout => "layout".to_owned(),
      ArchType::Page => "page".to_owned(),
      ArchType::Service => "service".to_owned(),
      ArchType::Schema => "schema".to_owned(),
      ArchType::Action => "action".to_owned(),
      ArchType::Store => "store".to_owned(),
      ArchType::Class => "class".to_owned(),
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
      Tool::React => "react".to_owned(),
      Tool::Svelte => "svelte".to_owned(),
      Tool::Vanilla => "vanilla".to_owned(),
    }
  }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Lang {
  JavaScript,
  TypeScript,
}

impl Lang {
  pub fn parse(name: &str) -> Lang {
    match name.to_lowercase().as_str() {
      "javascript" => Lang::JavaScript,
      "typescript" => Lang::TypeScript,
      _ => Lang::TypeScript,
    }
  }
  pub fn to_string(&self) -> String {
    match self {
      Lang::JavaScript => "javascript".to_owned(),
      Lang::TypeScript => "typescript".to_owned(),
    }
  }
  pub fn to_extension(&self) -> String {
    match self {
      Lang::JavaScript => ".js".to_owned(),
      Lang::TypeScript => ".ts".to_owned(),
    }
  }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Styles {
  CSS,
  SCSS,
  LESS,
  Stylus,
  PostCSS,
  Emotion,
  StyledComponents,
}

impl Styles {
  pub fn parse(name: &str) -> Styles {
    match name.to_lowercase().as_str() {
      "css" => Styles::CSS,
      "scss" => Styles::SCSS,
      "less" => Styles::LESS,
      "stylus" => Styles::Stylus,
      "postcss" => Styles::PostCSS,
      "emotion" => Styles::Emotion,
      "styled" => Styles::StyledComponents,
      _ => Styles::Emotion,
    }
  }
  pub fn to_string(&self) -> String {
    match self {
      Styles::CSS => "css".to_owned(),
      Styles::SCSS => "scss".to_owned(),
      Styles::LESS => "less".to_owned(),
      Styles::Stylus => "stylus".to_owned(),
      Styles::PostCSS => "postcss".to_owned(),
      Styles::Emotion => "emotion".to_owned(),
      Styles::StyledComponents => "styled".to_owned(),
    }
  }
  pub fn to_extension(&self, lang: &Lang) -> String {
    match self {
      Styles::CSS => ".css".to_owned(),
      Styles::SCSS => ".scss".to_owned(),
      Styles::LESS => ".less".to_owned(),
      Styles::Stylus => ".styl".to_owned(),
      Styles::PostCSS => ".postcss".to_owned(),
      Styles::Emotion => lang.to_extension(),
      Styles::StyledComponents => lang.to_extension(),
    }
  }
}
