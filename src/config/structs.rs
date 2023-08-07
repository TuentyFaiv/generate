#[derive(Clone, Debug)]
pub struct Paths {
  pub root: String,
  pub actions: String,
  pub stores: String,
  pub classes: String,
  pub functions: String,
  pub hocs: String,
  pub hooks: PathHooks,
  pub pages: String,
  pub layouts: String,
  pub ui: String,
  pub contexts: String,
  pub services: String,
  pub schemas: String,
  pub types: String,
  pub locales: PathLocales,
  pub routes: String,
}

impl Paths {
  pub fn get_root(&self, name: &str) -> String {
    format!("{}{name}", self.root)
  }
}

#[derive(Clone, Debug)]
pub struct PathLocales {
  pub react: String,
  pub svelte: String,
}

#[derive(Clone, Debug)]
pub struct PathHooks {
  pub global: String,
  pub internal: String,
}

#[derive(Clone, Debug)]
pub struct Tools {
  pub globals: Vec<String>,
  pub react: Vec<String>,
  pub svelte: Vec<String>,
  pub vanilla: Vec<String>,
  pub webcomponents: Vec<String>,
  pub components: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Archs {
  pub globals: Vec<String>,
  pub react: Vec<String>,
  pub svelte: Vec<String>,
  pub vanilla: Vec<String>,
}
