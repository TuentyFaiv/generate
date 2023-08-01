use crate::config::file::ConfigTemplates;

pub struct CreationPaths {
  pub template: String,
  pub default: String,
}

// Component
pub struct ComponentCreation {
  templates: Option<ConfigTemplates>,
  pub import: String,
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
  pub proptypes: Option<String>,
}

impl ComponentCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    import: String,
    styles: CreationPaths,
    component: CreationPaths,
    responsive: CreationPaths,
    proptypes: Option<CreationPaths>,
    script: Option<CreationPaths>,
    exports: ComponentCreationExports,
  ) -> Self {
    Self {
      templates: templates.clone(),
      import,
      styles,
      component,
      responsive,
      proptypes,
      script,
      exports
    }
  }

  pub fn react_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.react {
        None => None,
        Some(react) => react.component.clone()
      }
    }
  }
  pub fn svelte_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.svelte {
        None => None,
        Some(svelte) => svelte.component.clone()
      }
    }
  }
  pub fn vanilla_path(&self) -> Option<String> {
    match &self.templates {
      None => None,
      Some(templates) => match &templates.vanilla {
        None => None,
        Some(vanilla) => vanilla.component.clone()
      }
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
  pub proptypes: Option<CreationPaths>,
  pub i18n: Option<PageCreationI18n>,
  pub exports: PageCreationExports,
}

pub struct PageCreationImports {
  pub styles: String,
  pub page: Option<String>,
  pub i18n: Option<String>,
}

pub struct PageCreationAliases {
  pub ts_file: Option<String>,
  pub config: String,
}

pub struct PageCreationExports {
  pub page: String,
  pub styles: String,
  pub barrel_styles: String,
  pub responsive: String,
  pub locales: Option<Vec<String>>,
  pub i18n: Option<String>,
  pub proptypes: Option<String>,
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