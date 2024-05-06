#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Tool {
  // Frontend
  React,
  Svelte,
  Vanilla,
  Vue,
  Astro,
  // Native
  Swift,
  Kotlin,
  // Backend
  Node,
  Flask,
  FastAPI,
  Django,
  Actix,
  Rocket,
}

impl Tool {
  pub fn parse(name: &str) -> Tool {
    match name {
      "react" => Tool::React,
      "svelte" => Tool::Svelte,
      "vanilla" => Tool::Vanilla,
      "vue" => Tool::Vue,
      "astro" => Tool::Astro,
      "swift" => Tool::Swift,
      "kotlin" => Tool::Kotlin,
      "node" => Tool::Node,
      "flask" => Tool::Flask,
      "fastapi" => Tool::FastAPI,
      "django" => Tool::Django,
      "actix" => Tool::Actix,
      "rocket" => Tool::Rocket,
      _ => Tool::React,
    }
  }
  pub fn stringify(&self) -> &str {
    match self {
      Tool::React => "react",
      Tool::Svelte => "svelte",
      Tool::Vanilla => "vanilla",
      Tool::Vue => "vue",
      Tool::Astro => "astro",
      Tool::Swift => "swift",
      Tool::Kotlin => "kotlin",
      Tool::Node => "node",
      Tool::Flask => "flask",
      Tool::FastAPI => "fastapi",
      Tool::Django => "django",
      Tool::Actix => "actix",
      Tool::Rocket => "rocket",
    }
  }
  pub fn extension(&self) -> &str {
    match self {
      Tool::React => ".tsx",
      Tool::Svelte => ".svelte",
      Tool::Vanilla => ".js",
      Tool::Vue => ".vue",
      Tool::Astro => ".astro",
      Tool::Swift => ".swift",
      Tool::Kotlin => ".kt",
      Tool::Node => ".js",
      Tool::Flask => ".py",
      Tool::FastAPI => ".py",
      Tool::Django => ".py",
      Tool::Actix => ".rs",
      Tool::Rocket => ".rs",
    }
  }
  pub fn archs(&self) -> Vec<ArchType> {
    let shared_frontend = vec![
      ArchType::Project,
      ArchType::Library,
      ArchType::Component,
      ArchType::Service,
      ArchType::Schema,
      ArchType::Page,
      ArchType::Layout,
    ];
    let shared_native = vec![
      ArchType::Project,
      ArchType::Library,
      ArchType::Component,
      ArchType::Service,
      ArchType::Schema,
      ArchType::Page,
      ArchType::Layout,
    ];
    let shared_backend = vec![
      ArchType::Project,
      ArchType::Library,
      ArchType::Service,
      ArchType::Schema,
    ];

    match self {
      Tool::React => [shared_frontend, [
        ArchType::Hoc,
        ArchType::Hook,
        ArchType::Context,
      ].to_vec()].concat(),
      Tool::Svelte => [shared_frontend, [
        ArchType::Action,
        ArchType::Store,
      ].to_vec()].concat(),
      Tool::Vanilla => [shared_frontend, [
        ArchType::Class,
      ].to_vec()].concat(),
      Tool::Vue => shared_frontend,
      Tool::Astro => shared_frontend,
      Tool::Swift => shared_native,
      Tool::Kotlin => shared_native,
      Tool::Node => shared_backend,
      Tool::Flask => shared_backend,
      Tool::FastAPI => shared_backend,
      Tool::Django => shared_backend,
      Tool::Actix => shared_backend,
      Tool::Rocket => shared_backend,
    }
  }
  pub fn styles(&self) -> Vec<Styles> {
    let shared_styles = vec![
      Styles::CSS,
      Styles::SCSS,
      Styles::LESS,
      Styles::Stylus,
      Styles::ModuleCSS,
      Styles::PostCSS,
      Styles::Emotion,
    ];

    match self {
      Tool::React => [
        shared_styles,
        [Styles::StyledComponents].to_vec(),
      ].concat(),
      Tool::Svelte => shared_styles,
      Tool::Vanilla => shared_styles,
      Tool::Vue => shared_styles,
      Tool::Astro => shared_styles,
      Tool::Swift => vec![],
      Tool::Kotlin => vec![],
      Tool::Node => vec![],
      Tool::Flask => vec![],
      Tool::FastAPI => vec![],
      Tool::Django => vec![],
      Tool::Actix => vec![],
      Tool::Rocket => vec![],
    }
  }
  pub fn components(&self) -> Vec<Component> {
    let shared_components = vec![
      Component::Organism,
      Component::Molecule,
      Component::Atom,
      Component::Custom,
    ];
    match self {
      Tool::React => shared_components,
      Tool::Svelte => shared_components,
      Tool::Vanilla => shared_components,
      Tool::Vue => shared_components,
      Tool::Astro => shared_components,
      Tool::Swift => vec![],
      Tool::Kotlin => vec![],
      Tool::Node => vec![],
      Tool::Flask => vec![],
      Tool::FastAPI => vec![],
      Tool::Django => vec![],
      Tool::Actix => vec![],
      Tool::Rocket => vec![],
    }
  }
  pub fn langs(&self) -> Vec<Lang> {
    let shared_langs = vec![
      Lang::JavaScript,
      Lang::TypeScript,
    ];
    match self {
      Tool::React => shared_langs,
      Tool::Svelte => shared_langs,
      Tool::Vanilla => shared_langs,
      Tool::Vue => shared_langs,
      Tool::Astro => shared_langs,
      Tool::Node => shared_langs,
      Tool::Swift => vec![],
      Tool::Kotlin => vec![],
      Tool::Flask => vec![],
      Tool::FastAPI => vec![],
      Tool::Django => vec![],
      Tool::Actix => vec![],
      Tool::Rocket => vec![],
    }
  }
  pub fn all() -> Vec<Tool> {
    vec![
      Tool::React,
      Tool::Svelte,
      Tool::Vanilla,
      Tool::Vue,
      Tool::Astro,
      Tool::Swift,
      Tool::Kotlin,
      Tool::Node,
      Tool::Flask,
      Tool::FastAPI,
      Tool::Django,
      Tool::Actix,
      Tool::Rocket,
    ]
  }
}

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
  pub fn stringify(&self) -> &str {
    match self {
      ArchType::Project => "project",
      ArchType::Library => "library",
      ArchType::Component => "component",
      ArchType::Hoc => "hoc",
      ArchType::Hook => "hook",
      ArchType::Context => "context",
      ArchType::Layout => "layout",
      ArchType::Page => "page",
      ArchType::Service => "service",
      ArchType::Schema => "schema",
      ArchType::Action => "action",
      ArchType::Store => "store",
      ArchType::Class => "class",
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
  pub fn stringify(&self) -> &str {
    match self {
      Lang::JavaScript => "javascript",
      Lang::TypeScript => "typescript",
    }
  }
  pub fn extension(&self) -> &str {
    match self {
      Lang::JavaScript => ".js",
      Lang::TypeScript => ".ts",
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
  ModuleCSS,
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
      "modulecss" => Styles::ModuleCSS,
      _ => Styles::Emotion,
    }
  }
  pub fn stringify(&self) -> &str {
    match self {
      Styles::CSS => "css",
      Styles::SCSS => "scss",
      Styles::LESS => "less",
      Styles::Stylus => "stylus",
      Styles::PostCSS => "postcss",
      Styles::Emotion => "emotion",
      Styles::StyledComponents => "styled",
      Styles::ModuleCSS => "modulecss",
    }
  }
  pub fn extension<'a>(&self, lang: &'a Lang) -> &'a str {
    match self {
      Styles::CSS => ".css",
      Styles::SCSS => ".scss",
      Styles::LESS => ".less",
      Styles::Stylus => ".styl",
      Styles::PostCSS => ".postcss",
      Styles::ModuleCSS => ".module.css",
      Styles::Emotion => lang.extension(),
      Styles::StyledComponents => lang.extension(),
    }
  }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Component {
  Organism,
  Molecule,
  Atom,
  Custom,
}

impl Component {
  pub fn parse(name: &str) -> Component {
    match name.to_lowercase().as_str() {
      "organism" => Component::Organism,
      "molecule" => Component::Molecule,
      "atom" => Component::Atom,
      "custom" => Component::Custom,
      _ => Component::Custom,
    }
  }
  pub fn stringify(&self) -> &str {
    match self {
      Component::Organism => "organism",
      Component::Molecule => "molecule",
      Component::Atom => "atom",
      Component::Custom => "custom",
    }
  }
}
