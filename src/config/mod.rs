mod utils;
pub mod file;
pub mod structs;

use std::fs::File;
use std::io::BufReader;
use dirs_next::home_dir;

use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_VANILLA};
use crate::statics::{ARCHS, ARCHS_REACT, ARCHS_SVELTE, TOOLS_COMPONENTS, ARCHS_VANILLA};
use crate::statics::LANGS;
use crate::utils::to_vec;
use crate::cli::enums::Tool;
use crate::cli::structs::Args;

use self::utils::{tool_to_vec, to_tool_type, default_tool, list_libraries, list_projects, search_repository};
use self::file::{ConfigFile, ConfigFileToolsType, ConfigRepositoryTool, ConfigTemplates};
use self::structs::{Paths, PathLocales, Archs, Tools, PathHooks};

#[derive(Clone, Debug)]
pub struct CLIConfig {
  pub paths: Paths,
  pub langs: Vec<String>,
  pub tools: Tools,
  pub archs: Archs,
  pub templates: Option<ConfigTemplates>,
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
          println!("Home path: {:?}", path.to_str());
          CLIConfig::read_config(path.to_str().unwrap_or(own_path))
        },
        None => None
      },
    };

    println!("{:?}", config_file);

    Self {
      langs: to_vec(LANGS),
      paths: CLIConfig::build_paths(&config_file),
      tools: CLIConfig::build_tools(&config_file),
      tools_type: CLIConfig::build_tools_type(&config_file),
      repository: CLIConfig::build_repository(&config_file),
      archs: Archs {
        globals: to_vec(ARCHS),
        react: to_vec(ARCHS_REACT),
        svelte: to_vec(ARCHS_SVELTE),
        vanilla: to_vec(ARCHS_VANILLA),
      },
      templates: CLIConfig::build_templates(&config_file),
    }
  }
  fn read_config(path: &str) -> Option<ConfigFile> {
    println!("Reading config file: {}", path);
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
      actions: "./src/logic/actions".to_string(),
      stores: "./src/logic/stores".to_string(),
      classes: "./src/logic/classes".to_string(),
      functions: "./src/logic/functions".to_string(),
      hocs: "./src/logic/hocs".to_string(),
      hooks: PathHooks {
        global: "./src/logic/hooks".to_string(),
        internal: "hooks".to_string(),
      },
      pages: "./src/routes".to_string(),
      layouts: "./src/routes".to_string(),
      ui: "./src/ui".to_string(),
      contexts: "./src/logic/contexts".to_string(),
      services: "./src/logic/services".to_string(),
      schemas: "./src/logic/schemas".to_string(),
      types: "./src/logic/typing".to_string(),
      locales: PathLocales {
        react: "./src/ui/locales".to_string(),
        svelte: "./src/ui/locales".to_string(),
      },
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
          let services = match &paths.globals {
            Some(globals) => match &globals.services {
              Some(path) => path.clone(),
              None => default_paths.services
            },
            None => default_paths.services
          };
          let schemas = match &paths.globals {
            Some(globals) => match &globals.schemas {
              Some(path) => path.clone(),
              None => default_paths.schemas
            },
            None => default_paths.schemas
          };
          let contexts = match &paths.globals {
            Some(globals) => match &globals.contexts {
              Some(path) => path.clone(),
              None => default_paths.contexts
            },
            None => default_paths.contexts
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
          let hocs = match &paths.react {
            Some(react) => match &react.hocs {
              Some(path) => path.clone(),
              None => default_paths.hocs
            },
            None => default_paths.hocs
          };
          let hooks = match &paths.react {
            Some(react) => match &react.hooks {
              Some(path) => PathHooks {
                global: match &path.global {
                  Some(path) => path.clone(),
                  None => default_paths.hooks.global
                },
                internal: match &path.internal {
                  Some(path) => path.clone(),
                  None => default_paths.hooks.internal
                },
              },
              None => default_paths.hooks
            },
            None => default_paths.hooks
          };
          let react_locales = match &paths.react {
            Some(react) => match &react.locales {
              Some(path) => path.clone(),
              None => default_paths.locales.react
            },
            None => default_paths.locales.react
          };
          let routes = match &paths.react {
            Some(react) => match &react.routes {
              Some(path) => path.clone(),
              None => default_paths.routes
            },
            None => default_paths.routes
          };
          // Svelte paths
          let actions = match &paths.svelte {
            Some(svelte) => match &svelte.actions {
              Some(path) => path.clone(),
              None => default_paths.actions
            },
            None => default_paths.actions
          };
          let stores = match &paths.svelte {
            Some(svelte) => match &svelte.stores {
              Some(path) => path.clone(),
              None => default_paths.stores
            },
            None => default_paths.stores
          };
          let svelte_locales = match &paths.svelte {
            Some(svelte) => match &svelte.locales {
              Some(path) => path.clone(),
              None => default_paths.locales.svelte
            },
            None => default_paths.locales.svelte
          };
          let layouts = match &paths.svelte {
            Some(svelte) => match &svelte.layouts {
              Some(path) => path.clone(),
              None => default_paths.layouts
            },
            None => default_paths.layouts
          };
          let pages = match &paths.svelte {
            Some(svelte) => match &svelte.pages {
              Some(path) => path.clone(),
              None => default_paths.pages
            },
            None => default_paths.pages
          };
          // Vanilla paths
          let classes = match &paths.vanilla {
            Some(vanilla) => match &vanilla.classes {
              Some(path) => path.clone(),
              None => default_paths.classes
            },
            None => default_paths.classes
          };
          let functions = match &paths.vanilla {
            Some(vanilla) => match &vanilla.functions {
              Some(path) => path.clone(),
              None => default_paths.functions
            },
            None => default_paths.functions
          };

          Paths {
            root,
            actions,
            stores,
            classes,
            functions,
            hocs,
            hooks,
            pages,
            layouts,
            ui,
            contexts,
            services,
            schemas,
            types,
            locales: PathLocales {
              react: react_locales,
              svelte: svelte_locales,
            },
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
          let react = default_tool(&tools_type.react, default_tools.react);
          let svelte = default_tool(&tools_type.svelte, default_tools.svelte);
          let vanilla = default_tool(&tools_type.vanilla, default_tools.vanilla);

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
  fn build_templates(config_file: &Option<ConfigFile>) -> Option<ConfigTemplates> {
    match config_file {
      None => None,
      Some(file) => match &file.templates {
        None => None,
        Some(templates) => Some(templates.clone())
      }
    }
  }
  pub fn get_repository(&self, repo: &str) -> String {
    format!("{}/{}.git", self.repository, repo)
  }
  pub fn find_repository(&self, tool: &Tool, tool_type: &str) -> Option<ConfigRepositoryTool> {
    match tool {
      Tool::React => search_repository(&self.tools_type.react, tool_type),
      Tool::Svelte => search_repository(&self.tools_type.svelte, tool_type),
      Tool::Vanilla => search_repository(&self.tools_type.vanilla, tool_type),
    }
  }
  pub fn projects_options(&self, tool: &Tool) -> Vec<String> {
    match tool {
      Tool::React => list_projects(&self.tools_type.react),
      Tool::Svelte => list_projects(&self.tools_type.svelte),
      Tool::Vanilla => list_projects(&self.tools_type.vanilla),
    }
  }
  pub fn library_options(&self, tool: &Tool) -> Vec<String> {
    match tool {
      Tool::React => list_libraries(&self.tools_type.react),
      Tool::Svelte => list_libraries(&self.tools_type.svelte),
      Tool::Vanilla => list_libraries(&self.tools_type.vanilla),
    }
  }
}
