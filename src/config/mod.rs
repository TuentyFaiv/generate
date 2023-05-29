pub mod file;
pub mod structs;

use std::fs::File;
use std::io::BufReader;
use dirs_next::home_dir;

use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_VANILLA};
use crate::statics::{ARCHS, ARCHS_REACT, ARCHS_SVELTE, TOOLS_COMPONENTS, ARCHS_VANILLA};
use crate::statics::{LANGS};
use crate::utils::{to_vec, tool_to_vec, to_tool_type};
use crate::cli::enums::Tool;
use crate::cli::structs::Args;

use self::file::{ConfigFile, ConfigFileToolsType, ConfigRepositoryTool};
use self::structs::{Paths, Archs, Tools};

#[derive(Clone, Debug)]
pub struct CLIConfig {
  paths: Paths,
  langs: Vec<String>,
  tools: Tools,
  archs: Archs,
  tools_type: ConfigFileToolsType,
  repository: String,
}

impl CLIConfig {
  pub fn new(args: Args) -> Self {
    let config_file = match args.config {
      Some(config_path) => CLIConfig::read_config(&config_path),
      None => match home_dir() {
        Some(mut path) => {
          let own_path = ".tfverse/config_cli.json";
          path.push(own_path);
          CLIConfig::read_config(path.to_str().unwrap_or(own_path))
        },
        None => None
      },
    };

    let langs = to_vec(LANGS);
    let paths = CLIConfig::build_paths(&config_file);
    let tools = CLIConfig::build_tools(&config_file);
    let tools_type = CLIConfig::build_tools_type(&config_file);
    let repository = CLIConfig::build_repository(&config_file);
    let archs = Archs {
      globals: to_vec(ARCHS),
      react: to_vec(ARCHS_REACT),
      svelte: to_vec(ARCHS_SVELTE),
      vanilla: to_vec(ARCHS_VANILLA),
    };

    Self { paths, langs, tools, tools_type, archs, repository }
  }
  fn read_config(path: &str) -> Option<ConfigFile> {
    match File::open(path) {
      Err(_) => None,
      Ok(file) => {
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, ConfigFile>(reader) {
          Err(_) => None,
          Ok(config) => Some(config),
        }
      },
    }
  }
  fn build_paths(config_file: &Option<ConfigFile>) -> Paths {
    let default_paths = Paths {
      root: "./".to_string(),
      action: "./src/logic/actions".to_string(),
      store: "./src/logic/stores".to_string(),
      class: "./src/logic/classes".to_string(),
      function: "./src/logic/functions".to_string(),
      hoc: "./src/logic/hocs".to_string(),
      hook: "./src/logic/hooks".to_string(),
      page: "./src/routes".to_string(),
      layout: "./src/routes".to_string(),
      ui: "./src/ui".to_string(),
      context: "./src/logic/contexts".to_string(),
      service: "./src/logic/services".to_string(),
      schema: "./src/logic/schemas".to_string(),
      types: "./src/logic/typing".to_string(),
      svelte_locales: "./static/locales".to_string(),
      react_locales: "./public/locales".to_string(),
      routes: "./src/logic/routes".to_string(),
    };
    match config_file.as_ref() {
      Some(file) => match &file.paths {
        Some(paths) => {
          let root = match &file.root {
            Some(root) => root.clone(),
            None => default_paths.root
          };
          // Global paths
          let service = match &paths.globals {
            Some(globals) => match &globals.service {
              Some(path) => path.clone(),
              None => default_paths.service
            },
            None => default_paths.service
          };
          let schema = match &paths.globals {
            Some(globals) => match &globals.schema {
              Some(path) => path.clone(),
              None => default_paths.schema
            },
            None => default_paths.schema
          };
          let context = match &paths.globals {
            Some(globals) => match &globals.context {
              Some(path) => path.clone(),
              None => default_paths.context
            },
            None => default_paths.context
          };
          let types = match &paths.globals {
            Some(globals) => match &globals.types {
              Some(path) => path.clone(),
              None => default_paths.types
            },
            None => default_paths.types
          };
          let ui = match &paths.globals {
            Some(globals) => match &globals.ui {
              Some(path) => path.clone(),
              None => default_paths.ui
            },
            None => default_paths.ui
          };
          // React paths
          let hoc = match &paths.react {
            Some(react) => match &react.hoc {
              Some(path) => path.clone(),
              None => default_paths.hoc
            },
            None => default_paths.hoc
          };
          let hook = match &paths.react {
            Some(react) => match &react.hook {
              Some(path) => path.clone(),
              None => default_paths.hook
            },
            None => default_paths.hook
          };
          let react_locales = match &paths.react {
            Some(react) => match &react.locales {
              Some(path) => path.clone(),
              None => default_paths.react_locales
            },
            None => default_paths.react_locales
          };
          let routes = match &paths.react {
            Some(react) => match &react.routes {
              Some(path) => path.clone(),
              None => default_paths.routes
            },
            None => default_paths.routes
          };
          // Svelte paths
          let action = match &paths.svelte {
            Some(svelte) => match &svelte.action {
              Some(path) => path.clone(),
              None => default_paths.action
            },
            None => default_paths.action
          };
          let store = match &paths.svelte {
            Some(svelte) => match &svelte.store {
              Some(path) => path.clone(),
              None => default_paths.store
            },
            None => default_paths.store
          };
          let svelte_locales = match &paths.svelte {
            Some(svelte) => match &svelte.locales {
              Some(path) => path.clone(),
              None => default_paths.svelte_locales
            },
            None => default_paths.svelte_locales
          };
          let layout = match &paths.svelte {
            Some(svelte) => match &svelte.layout {
              Some(path) => path.clone(),
              None => default_paths.layout
            },
            None => default_paths.layout
          };
          let page = match &paths.svelte {
            Some(svelte) => match &svelte.page {
              Some(path) => path.clone(),
              None => default_paths.page
            },
            None => default_paths.page
          };
          // Vanilla paths
          let class = match &paths.vanilla {
            Some(vanilla) => match &vanilla.class {
              Some(path) => path.clone(),
              None => default_paths.class
            },
            None => default_paths.class
          };
          let function = match &paths.vanilla {
            Some(vanilla) => match &vanilla.function {
              Some(path) => path.clone(),
              None => default_paths.function
            },
            None => default_paths.function
          };

          Paths {
            root,
            action,
            store,
            class,
            function,
            hoc,
            hook,
            page,
            layout,
            ui,
            context,
            service,
            schema,
            types,
            svelte_locales,
            react_locales,
            routes,
          }
        },
        None => default_paths
      }
      None => default_paths
    }
  }
  fn build_repository(config_file: &Option<ConfigFile>) -> String {
    let default_repo = "git@github.com:TuentyFaiv".to_string();
    match config_file {
      None => default_repo,
      Some(file) => match &file.repository {
        None => default_repo,
        Some(repo) => repo.replace(".git", "")
      }
    }
  }
  fn build_tools(config_file: &Option<ConfigFile>) -> Tools {
    let react_tools = tool_to_vec(TOOLS_REACT);
    let svelte_tools = tool_to_vec(TOOLS_SVELTE);
    let vanilla_tools = tool_to_vec(TOOLS_VANILLA);
    let default_tools = Tools {
      globals: to_vec(TOOLS),
      react: react_tools,
      svelte: svelte_tools,
      vanilla: vanilla_tools,
      webcomponents: to_vec(TOOLS_WEBCOMPONENTS),
      components: to_vec(TOOLS_COMPONENTS),
    };

    match config_file {
      None => default_tools,
      Some(file) => match &file.tools_type {
        None => default_tools,
        Some(tools_type) => {
          let react = match &tools_type.react {
            Some(react) => {
              react.iter().map(|(tool, _)| tool.clone()).collect()
            },
            None => default_tools.react
          };
          let svelte = match &tools_type.svelte {
            Some(svelte) => {
              svelte.iter().map(|(tool, _)| tool.clone()).collect()
            },
            None => default_tools.svelte
          };
          let vanilla = match &tools_type.vanilla {
            Some(vanilla) => {
              vanilla.iter().map(|(tool, _)| tool.clone()).collect()
            },
            None => default_tools.vanilla
          };

          Tools {
            globals: default_tools.globals,
            react,
            svelte,
            vanilla,
            webcomponents: default_tools.webcomponents,
            components: default_tools.components,
          }
        }
      }
    }
  }
  fn build_tools_type(config_file: &Option<ConfigFile>) -> ConfigFileToolsType {
    let default_tools = ConfigFileToolsType {
      react: to_tool_type(TOOLS_REACT),
      svelte: to_tool_type(TOOLS_SVELTE),
      vanilla: to_tool_type(TOOLS_VANILLA),
    };

    match config_file {
      None => default_tools,
      Some(file) => match &file.tools_type {
        None => default_tools,
        Some(tools) => {
          let react = match &tools.react {
            Some(react) => Some(react.clone()),
            None => default_tools.react
          };
          let svelte = match &tools.svelte {
            Some(svelte) => Some(svelte.clone()),
            None => default_tools.svelte
          };
          let vanilla = match &tools.vanilla {
            Some(vanilla) => Some(vanilla.clone()),
            None => default_tools.vanilla
          };

          ConfigFileToolsType {
            react,
            svelte,
            vanilla,
          }
        }
      }
    }
  }
  pub fn get_paths(&self) -> &Paths {
    &self.paths
  }
  pub fn get_tools(&self) -> &Tools {
    &self.tools
  }
  pub fn get_archs(&self) -> &Archs {
    &self.archs
  }
  pub fn get_langs(&self) -> &Vec<String> {
    &self.langs
  }
  pub fn get_repository(&self, repo: &str) -> String {
    format!("{}/{}.git", self.repository, repo)
  }
  pub fn find_repository(&self, tool: &Tool, tool_type: &str) -> Option<ConfigRepositoryTool> {
    match tool {
      Tool::React => {
        self.tools_type.react.clone()?.iter()
          .find(|(name, _)| name.as_str() == tool_type)?.1.clone()
      },
      Tool::Svelte => {
        self.tools_type.svelte.clone()?.iter()
          .find(|(name, _)| name.as_str() == tool_type)?.1.clone()
      },
      Tool::Vanilla => {
        self.tools_type.vanilla.clone()?.iter()
          .find(|(name, _)| name.as_str() == tool_type)?.1.clone()
      },
    }
  }
  pub fn projects_options(&self, tool: &Tool) -> Vec<String> {
    match tool {
      Tool::React => match &self.tools_type.react {
        None => [].to_vec(),
        Some(react) => react.iter().map(|(name, _)| name.clone()).collect()
      },
      Tool::Svelte => match &self.tools_type.svelte {
        None => [].to_vec(),
        Some(svelte) => svelte.iter().map(|(name, _)| name.clone()).collect()
      },
      Tool::Vanilla => match &self.tools_type.vanilla {
        None => [].to_vec(),
        Some(vanilla) => vanilla.iter().map(|(name, _)| name.clone()).collect()
      },
    }
  }
  pub fn library_options(&self, tool: &Tool) -> Vec<String> {
    match tool {
      Tool::React => match &self.tools_type.react {
        None => [].to_vec(),
        Some(react) => react.iter().filter_map(|(name, repository)| -> Option<String> {
          match repository {
            Some(repo) => match repo.library {
              Some(_) => Some(name.clone()),
              None => None
            },
            None => None
          }
        }).collect()
      },
      Tool::Svelte => match &self.tools_type.svelte {
        None => [].to_vec(),
        Some(svelte) => svelte.iter().filter_map(|(name, repository)| -> Option<String> {
          match repository {
            Some(repo) => match repo.library {
              Some(_) => Some(name.clone()),
              None => None
            },
            None => None
          }
        }).collect()
      },
      Tool::Vanilla => match &self.tools_type.vanilla {
        None => [].to_vec(),
        Some(vanilla) => vanilla.iter().filter_map(|(name, repository)| -> Option<String> {
          match repository {
            Some(repo) => match repo.library {
              Some(_) => Some(name.clone()),
              None => None
            },
            None => None
          }
        }).collect()
      },
    }
  }
}
