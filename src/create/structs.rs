use crate::config::file::ConfigTemplates;

pub struct ComponentCreation {
  templates: Option<ConfigTemplates>,
  pub import: String,
  pub styles: ComponentCreationPaths,
  pub component: ComponentCreationPaths,
  pub responsive: ComponentCreationPaths,
  pub proptypes: Option<ComponentCreationPaths>,
  pub script: Option<ComponentCreationPaths>,
  pub exports: ComponentExports,
}

pub struct ComponentCreationPaths {
  pub template: String,
  pub default: String,
}

pub struct ComponentExports {
  pub styles: String,
  pub responsive: String,
  pub component: String,
  pub proptypes: Option<String>,
}

impl ComponentCreation {
  pub fn new(
    templates: &Option<ConfigTemplates>,
    import: String,
    styles: ComponentCreationPaths,
    component: ComponentCreationPaths,
    responsive: ComponentCreationPaths,
    proptypes: Option<ComponentCreationPaths>,
    script: Option<ComponentCreationPaths>,
    exports: ComponentExports,
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