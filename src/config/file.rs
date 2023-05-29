use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
  pub repository: Option<String>,
  pub root: Option<String>,
  pub paths: Option<ConfigPaths>,
  pub tools_type: Option<ConfigFileToolsType>,
  pub templates: Option<ConfigTemplates>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigPaths {
  pub globals: Option<ConfigGlobalPaths>,
  pub react: Option<ConfigReactPaths>,
  pub svelte: Option<ConfigSveltePaths>,
  pub vanilla: Option<ConfigVanillaPaths>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigGlobalPaths {
  pub service: Option<String>,
  pub schema: Option<String>,
  pub context: Option<String>,
  pub types: Option<String>,
  pub ui: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigReactPaths {
  pub hoc: Option<String>,
  pub hook: Option<String>,
  pub locales: Option<String>,
  pub routes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSveltePaths {
  pub action: Option<String>,
  pub store: Option<String>,
  pub page: Option<String>,
  pub layout: Option<String>,
  pub locales: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigVanillaPaths {
  pub class: Option<String>,
  pub function: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RepositoryTool<'a> {
  pub name: &'a str,
  pub project: &'a str,
  pub library: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct ConfigToolsType<'a> {
  pub react: Vec<(&'a str, Option<RepositoryTool<'a>>)>,
  pub svelte: Vec<(&'a str, Option<RepositoryTool<'a>>)>,
  pub vanilla: Vec<(&'a str, Option<RepositoryTool<'a>>)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigRepositoryTool {
  pub name: Option<String>,
  pub project: Option<String>,
  pub library: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigFileToolsType {
  pub react: Option<Vec<(String, Option<ConfigRepositoryTool>)>>,
  pub svelte: Option<Vec<(String, Option<ConfigRepositoryTool>)>>,
  pub vanilla: Option<Vec<(String, Option<ConfigRepositoryTool>)>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigTemplates {
  pub react: Option<ConfigReactTemplates>,
  pub svelte: Option<ConfigSvelteTemplates>,
  pub vanilla: Option<ConfigVanillaTemplates>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigReactTemplates {
  pub service: Option<String>,
  pub schema: Option<String>,
  pub hoc: Option<String>,
  pub hook: Option<String>,
  pub context: Option<String>,
  pub page: Option<String>,
  pub layout: Option<String>,
  pub component: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSvelteTemplates {
  pub service: Option<String>,
  pub schema: Option<String>,
  pub action: Option<String>,
  pub store: Option<String>,
  pub context: Option<String>,
  pub page: Option<String>,
  pub layout: Option<String>,
  pub component: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigVanillaTemplates {
  pub service: Option<String>,
  pub schema: Option<String>,
  pub class: Option<String>,
  pub function: Option<String>,
  pub page: Option<String>,
  pub layout: Option<String>,
  pub component: Option<String>,
}