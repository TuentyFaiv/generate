mod utils;
mod global;
pub mod file;
pub mod structs;

use std::fs::{File, create_dir_all};
use std::io::{Write, BufWriter, BufReader};
use anyhow::{Result, anyhow};
use dirs_next::home_dir;

use crate::statics::{TOOLS_REACT, TOOLS_SVELTE, TOOLS_VANILLA};
use crate::technologies::enums::{Tool, Lang, Styles};
use crate::cli::structs::Args;

use self::global::{i18n_question, styles_question, lang_question, repository_question};
use self::utils::{to_tool_type, list_libraries, list_projects, search_repository};
use self::file::{ConfigFile, ConfigFileToolsType, ConfigRepositoryTool, ConfigTemplates};
use self::structs::{Paths, PathLocales, PathHooks, ConfigStored};

#[derive(Clone, Debug)]
pub struct CLIConfig {
  pub stored: Option<ConfigStored>,
  pub paths: Paths,
  pub templates: Option<ConfigTemplates>,
  tools_type: ConfigFileToolsType,
  repository: String,
}

const OWN_PATH: &str = ".tfverse/config_cli.json";

impl CLIConfig {
  pub fn new(args: Args) -> Result<Self> {
    let config_file = match args.config {
      Some(config_path) => CLIConfig::read_config(&config_path),
      None => CLIConfig::read_global(),
    };

    Ok(Self {
      stored: CLIConfig::set_storage(args.global)?,
      paths: CLIConfig::build_paths(&config_file),
      tools_type: CLIConfig::build_tools_type(&config_file),
      repository: CLIConfig::build_repository(&config_file),
      templates: CLIConfig::build_templates(&config_file),
    })
  }
  fn read_global() -> Option<ConfigFile> {
    match home_dir() {
      Some(mut path) => {
        path.push(OWN_PATH);
        CLIConfig::read_config(path.to_str().unwrap_or(OWN_PATH))
      },
      None => None
    }
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
  fn set_storage(set_global: bool) -> Result<Option<ConfigStored>> {
    let full_path = match home_dir() {
      Some(mut full_path) => {
        full_path.push(OWN_PATH);
        full_path
      },
      None => return Err(anyhow!("Error to find HOME path")),
    };
    let dir_path = full_path.to_str()
      .unwrap_or(OWN_PATH)
      .to_string().replace("/config_cli.json", "");
    let config_file = CLIConfig::read_global();

    let storaged: Result<Option<ConfigStored>> = match &config_file {
      Some(file) => {
        let storaged_i18n = match &config_file {
          Some(file) => match file.i18n {
            Some(i18n) => if set_global { i18n_question()? } else { i18n },
            None => i18n_question()?,
          },
          None => i18n_question()?,
        };
        let storaged_lang = match &config_file {
          Some(file) => match &file.lang {
            Some(lang) => if set_global { lang_question()? } else { Lang::parse(&lang) },
            None => lang_question()?,
          },
          None => lang_question()?,
        };
        let storaged_styles = match &config_file {
          Some(file) => match &file.styles {
            Some(styles) => if set_global { styles_question()? } else { Styles::parse(&styles) },
            None => styles_question()?,
          },
          None => styles_question()?,
        };
        let storaged_repository = match &config_file {
          Some(file) => match &file.repository {
            Some(repository) => if set_global { repository_question()? } else { repository.to_string() },
            None => repository_question()?
          },
          None => repository_question()?
        };

        if set_global {
          let global_config = ConfigFile {
            i18n: Some(storaged_i18n),
            lang: Some(storaged_lang.stringify().to_string()),
            styles: Some(storaged_styles.stringify().to_string()),
            repository: Some(storaged_repository),
            paths: file.paths.clone(),
            root: file.root.clone(),
            templates: file.templates.clone(),
            tools_type: file.tools_type.clone(),
          };
    
          let file = File::create(&full_path)?;
          let mut buf_writer = BufWriter::new(file);
          serde_json::to_writer_pretty(&mut buf_writer, &global_config)?;
          buf_writer.flush()?;
        }

        Ok(Some(ConfigStored { i18n: storaged_i18n, lang: storaged_lang, styles: storaged_styles }))
      },
      None => {
        if set_global {
          create_dir_all(dir_path).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
          });
  
          let file = File::create(&full_path)?;
          let mut buf_writer = BufWriter::new(file);
          serde_json::to_writer_pretty(&mut buf_writer, &ConfigFile {
            repository: None,
            i18n: None,
            lang: None,
            styles: None,
            paths: None,
            root: None,
            templates: None,
            tools_type: None,
          })?;
          buf_writer.flush()?;
        }
        Ok(if set_global {CLIConfig::set_storage(set_global)? } else { None })
      },
    };

    Ok(storaged?)
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
        react: "./public/locales".to_string(),
        svelte: "./static/locales".to_string(),
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
      _ => None,
    }
  }
  pub fn projects_options(&self, tool: &Tool) -> Vec<String> {
    match tool {
      Tool::React => list_projects(&self.tools_type.react),
      Tool::Svelte => list_projects(&self.tools_type.svelte),
      Tool::Vanilla => list_projects(&self.tools_type.vanilla),
      _ => vec![],
    }
  }
  pub fn library_options(&self, tool: &Tool) -> Vec<String> {
    match tool {
      Tool::React => list_libraries(&self.tools_type.react),
      Tool::Svelte => list_libraries(&self.tools_type.svelte),
      Tool::Vanilla => list_libraries(&self.tools_type.vanilla),
      _ => vec![],
    }
  }
}
