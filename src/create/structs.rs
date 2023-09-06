use crate::{config::file::ConfigTemplates, cli::enums::Tool};

pub struct CreationPaths {
  pub template: String,
  pub default: String,
}

// Component
pub struct ComponentCreation {
  templates: Option<ConfigTemplates>,
  pub styles_ext: String,
  pub import: CreationPaths,
  pub styles: CreationPaths,
  pub component: CreationPaths,
  pub responsive: CreationPaths,
  pub proptypes: Option<CreationPaths>,
  pub script: Option<CreationPaths>,
  pub exports: ComponentCreationExports,
}

pub struct ComponentCreationExports {
  pub styles: String,
  pub responsive: String,
  pub component: String,
  pub barrel: String,
  pub proptypes: Option<String>,
}

impl ComponentCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    styles_ext: String,
    import: CreationPaths,
    styles: CreationPaths,
    component: CreationPaths,
    responsive: CreationPaths,
    proptypes: Option<CreationPaths>,
    script: Option<CreationPaths>,
    exports: ComponentCreationExports,
  ) -> Self {
    Self {
      templates: templates.clone(),
      styles_ext,
      import,
      styles,
      component,
      responsive,
      proptypes,
      script,
      exports
    }
  }

  pub fn path(&self, tool: &Tool) -> Option<String> {
    match &self.templates {
      Some(templates) => match tool {
        Tool::React => match &templates.react {
          None => None,
          Some(react) => react.component.clone()
        },
        Tool::Svelte => match &templates.svelte {
          None => None,
          Some(svelte) => svelte.component.clone()
        },
        Tool::Vanilla => match &templates.vanilla {
          None => None,
          Some(vanilla) => vanilla.component.clone()
        },
      },
      None => None,
    }
  }
}

// Page
pub struct PageCreation {
  templates: Option<ConfigTemplates>,
  pub imports: PageCreationImports,
  pub page: CreationPaths,
  pub styles: CreationPaths,
  pub responsive: CreationPaths,
  pub aliases: PageCreationAliases,
  pub router: Option<CreationPaths>,
  pub route: Option<CreationPaths>,
  pub script: Option<CreationPaths>,
  pub proptypes: Option<CreationPaths>,
  pub i18n: Option<PageCreationI18n>,
  pub exports: PageCreationExports,
}

pub struct PageCreationImports {
  pub styles: String,
  pub page: Option<String>,
  pub locale: Option<String>,
  pub i18n: Option<String>,
}

pub struct PageCreationAliases {
  pub ts_file: Option<CreationPaths>,
  pub ts_aliases: Option<CreationPaths>,
  pub config: CreationPaths,
  pub config_aliases: CreationPaths,
}

pub struct PageCreationExports {
  pub page: String,
  pub styles: String,
  pub barrel_styles: String,
  pub responsive: String,
  pub config: String,
  pub barrel_i18n: Option<String>,
  pub locales: Option<Vec<String>>,
  pub i18n: Option<String>,
  pub proptypes: Option<String>,
  pub router: Option<String>,
}

pub struct PageCreationI18n {
  pub locale: CreationPaths,
  pub context: CreationPaths,
}

impl PageCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    imports: PageCreationImports,
    page: CreationPaths,
    styles: CreationPaths,
    responsive: CreationPaths,
    aliases: PageCreationAliases,
    router: Option<CreationPaths>,
    route: Option<CreationPaths>,
    script: Option<CreationPaths>,
    proptypes: Option<CreationPaths>,
    i18n: Option<PageCreationI18n>,
    exports: PageCreationExports,
  ) -> Self {
    Self {
      templates: templates.clone(),
      imports,
      page,
      styles,
      responsive,
      aliases,
      script,
      router,
      route,
      proptypes,
      i18n,
      exports
    }
  }

  pub fn react_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.react {
        None => None,
        Some(react) => react.page.clone()
      }
    }
  }
  pub fn svelte_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.svelte {
        None => None,
        Some(svelte) => svelte.page.clone()
      }
    }
  }
  pub fn vanilla_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.vanilla {
        None => None,
        Some(vanilla) => vanilla.page.clone()
      }
    }
  }
}

// Layout
pub struct LayoutCreation {
  templates: Option<ConfigTemplates>,
  pub import: String,
  pub layout: CreationPaths,
  pub styles: CreationPaths,
  pub responsive: CreationPaths,
  pub proptypes: Option<CreationPaths>,
  pub script: Option<CreationPaths>,
  pub exports: LayoutCreationExports,
}

pub struct LayoutCreationExports {
  pub barrel_styles: String,
  pub layout: String,
  pub styles: String,
  pub responsive: String,
  pub proptypes: Option<String>,
}

impl LayoutCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    import: String,
    layout: CreationPaths,
    styles: CreationPaths,
    responsive: CreationPaths,
    proptypes: Option<CreationPaths>,
    script: Option<CreationPaths>,
    exports: LayoutCreationExports,
  ) -> Self {
    Self {
      templates: templates.clone(),
      import,
      layout,
      styles,
      responsive,
      proptypes,
      script,
      exports,
    }
  }

  pub fn react_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.react {
        None => None,
        Some(react) => react.layout.clone()
      }
    }
  }
  pub fn svelte_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.svelte {
        None => None,
        Some(svelte) => svelte.layout.clone()
      }
    }
  }
  pub fn vanilla_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.vanilla {
        None => None,
        Some(vanilla) => vanilla.layout.clone()
      }
    }
  }
}

// Schema
pub struct SchemaCreation {
  templates: Option<ConfigTemplates>,
  pub import: SchemaCreationImports,
  pub schema: CreationPaths,
  pub proptypes: Option<CreationPaths>,
  pub proptypes_imports: Option<CreationPaths>,
  pub exports: SchemaCreationExports,
}

pub struct SchemaCreationImports {
  pub barrel: String,
  pub types: String,
}

pub struct SchemaCreationExports {
  pub barrel: String,
  pub schema: String,
  pub proptypes: Option<String>,
}

impl SchemaCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    import: SchemaCreationImports,
    schema: CreationPaths,
    proptypes: Option<CreationPaths>,
    proptypes_imports: Option<CreationPaths>,
    exports: SchemaCreationExports,
  ) -> Self {
    Self {
      templates: templates.clone(),
      import,
      schema,
      proptypes,
      proptypes_imports,
      exports,
    }
  }

  pub fn react_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.react {
        None => None,
        Some(react) => react.schema.clone()
      }
    }
  }
  pub fn svelte_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.svelte {
        None => None,
        Some(svelte) => svelte.schema.clone()
      }
    }
  }
  pub fn vanilla_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.vanilla {
        None => None,
        Some(vanilla) => vanilla.schema.clone()
      }
    }
  }
}

// Schema
pub struct ServiceCreation {
  templates: Option<ConfigTemplates>,
  pub import: ServiceCreationImports,
  pub service: CreationPaths,
  pub instances: CreationPaths,
  pub proptypes: Option<CreationPaths>,
  pub proptypes_imports: Option<CreationPaths>,
  pub exports: ServiceCreationExports,
}

pub struct ServiceCreationImports {
  pub barrel: String,
  pub barrel_instances: String,
}

pub struct ServiceCreationExports {
  pub barrel: String,
  pub barrel_instances: String,
  pub service: String,
  pub instances: String,
  pub proptypes: Option<String>,
}

impl ServiceCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    import: ServiceCreationImports,
    service: CreationPaths,
    instances: CreationPaths,
    proptypes: Option<CreationPaths>,
    proptypes_imports: Option<CreationPaths>,
    exports: ServiceCreationExports,
  ) -> Self {
    Self {
      templates: templates.clone(),
      import,
      service,
      instances,
      proptypes,
      proptypes_imports,
      exports,
    }
  }

  pub fn react_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.react {
        None => None,
        Some(react) => react.service.clone()
      }
    }
  }
  pub fn svelte_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.svelte {
        None => None,
        Some(svelte) => svelte.service.clone()
      }
    }
  }
  pub fn vanilla_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.vanilla {
        None => None,
        Some(vanilla) => vanilla.service.clone()
      }
    }
  }
}