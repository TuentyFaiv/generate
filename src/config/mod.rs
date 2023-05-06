use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_BASE};
use crate::statics::{ARCHS, ARCHS_REACT, ARCHS_SVELTE, ARCHS_TYPE_COMPONENT, ARCHS_VANILLA};

#[derive(Debug)]
pub struct Paths<'a> {
  pub root: &'a str,
  pub action: &'a str,
  pub store: &'a str,
  pub class: &'a str,
  pub hoc: &'a str,
  pub hook: &'a str,
  pub page: &'a str,
  pub layout: &'a str,
  pub ui: &'a str,
  pub context: &'a str,
  pub service: &'a str,
  pub schema: &'a str,
  pub types: &'a str,
  pub svelte_locales: &'a str,
  pub react_locales: &'a str,
  pub routes: &'a str,
}

impl<'a> Paths<'a> {
  pub fn get_root(&self, name: &str) -> String {
    format!("{}{name}", self.root)
  }
}

#[derive(Debug)]
pub struct Tools<'a> {
  pub globals: &'a [&'a str],
  pub react: &'a [&'a str],
  pub svelte: &'a [&'a str],
  pub vanilla: &'a [&'a str],
  pub webcomponents: &'a [&'a str],
}

#[derive(Debug)]
pub struct Archs<'a> {
  pub globals: &'a [&'a str],
  pub react: &'a [&'a str],
  pub svelte: &'a [&'a str],
  pub vanilla: &'a [&'a str],
  pub type_component: &'a [&'a str],
}

pub trait Config {
  fn get_paths(&self) -> &Paths;
  fn get_tools(&self) -> &Tools;
  fn get_archs(&self) -> &Archs;
}


pub struct CLIConfig<'a> {
  paths: Paths<'a>,
  tools: Tools<'a>,
  archs: Archs<'a>,
}

impl<'a> CLIConfig<'a> {
  pub fn new() -> Self {
    let paths = Paths {
      root: "./",
      action: "./src/logic/actions",
      store: "./src/logic/stores",
      class: "./src/logic/classes",
      hoc: "./src/logic/hocs",
      hook: "./src/logic/hooks",
      page: "./src/routes",
      layout: "./src/routes",
      ui: "./src/ui",
      context: "./src/logic/contexts",
      service: "./src/logic/services",
      schema: "./src/logic/schemas",
      types: "./src/logic/typing",
      svelte_locales: "./static/locales",
      react_locales: "./public/locales",
      routes: "./src/logic/routes",
    };
    let tools = Tools {
      globals: TOOLS,
      react: TOOLS_REACT,
      svelte: TOOLS_SVELTE,
      vanilla: TOOLS_BASE,
      webcomponents: TOOLS_WEBCOMPONENTS,
    };
    let archs = Archs {
      globals: ARCHS,
      react: ARCHS_REACT,
      svelte: ARCHS_SVELTE,
      vanilla: ARCHS_VANILLA,
      type_component: ARCHS_TYPE_COMPONENT,
    };
    Self { paths, tools, archs }
  } 
}

impl<'a> Config for CLIConfig<'a> {
  fn get_paths(&self) -> &Paths {
    &self.paths
  }
  fn get_tools(&self) -> &Tools {
    &self.tools
  }
  fn get_archs(&self) -> &Archs {
    &self.archs
  }
}
