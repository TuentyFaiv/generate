#[derive(Clone, Debug)]
pub struct Paths {
  pub root: String,
  pub action: String,
  pub store: String,
  pub class: String,
  pub function: String,
  pub hoc: String,
  pub hook: String,
  pub page: String,
  pub layout: String,
  pub ui: String,
  pub context: String,
  pub service: String,
  pub schema: String,
  pub types: String,
  pub svelte_locales: String,
  pub react_locales: String,
  pub routes: String,
}

impl Paths {
  pub fn get_root(&self, name: &str) -> String {
    format!("{}{name}", self.root)
  }
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
