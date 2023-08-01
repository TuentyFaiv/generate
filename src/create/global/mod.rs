use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::create::structs::{PageCreationAliases, PageCreationI18n};
use crate::statics::OK;
use crate::templates;
use crate::utils::{change_case, transform};
use crate::config::CLIConfig;
use crate::cli::{utils::done, enums::Tool, structs::Answers};
use crate::templates::global::CLIGlobalTemplates;
use crate::templates::react::CLIReactTemplates;
use crate::templates::svelte::CLISvelteTemplates;

use super::structs::{
  CreationPaths,
  ComponentCreation,
  ComponentCreationExports,
  PageCreation,
  PageCreationImports,
  PageCreationExports,
};

pub struct CLIGlobalCreation {
  answers: Answers,
  config: CLIConfig,
  global: CLIGlobalTemplates,
  react: CLIReactTemplates,
  svelte: CLISvelteTemplates,
  error: String,
}

impl CLIGlobalCreation {
  pub fn new(config: CLIConfig, answers: Answers, error: String) -> Self {
    let global = CLIGlobalTemplates::new(config.clone(), answers.clone());
    let react = CLIReactTemplates::new(config.clone(), answers.clone());
    let svelte = CLISvelteTemplates::new(config.clone(), answers.clone());

    Self { config, answers, global, react, svelte, error }
  }
  pub fn make_component(&self) -> Result<String> {
    let Answers { name, path, tool, tool_type, language, .. } = &self.answers;
    let is_ts = language.as_str() == "typescript";
    let ext = if is_ts { ".ts".to_string() } else { ".js".to_string() };

    let full_path = match tool_type {
      Some(tool_type) => format!("{path}/{tool_type}/{name}"),
      None => format!("{path}/{name}")
    };

    if tool != &Tool::Vanilla {
      create_dir_all(&full_path).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    let component = match tool {
      Tool::React => {
        use templates::react::statics::component::{
          COMPONENT,
          COMPONENT_TS,
          PROPTYPES,
          STYLES,
          STYLES_RESPONSIVE,
        };
        
        let proptypes = if is_ts {
          Some(CreationPaths {
            template: format!("/proptypes{ext}"),
            default: PROPTYPES.to_string(),
          })
        } else { None };

        Ok(ComponentCreation::new(
          &self.config.templates,
          format!("export {{ default as {name} }} from \"./{name}/{name}\";\n"),
          CreationPaths {
            template: format!("/styles{ext}"),
            default: STYLES.to_string(),
          },
          CreationPaths {
            template: format!("/component{ext}x"),
            default: if is_ts { COMPONENT_TS.to_string() } else { COMPONENT.to_string() },
          },
          CreationPaths {
            template: format!("/styles.responsive{ext}"),
            default: STYLES_RESPONSIVE.to_string(),
          },
          proptypes,
          None,
          ComponentCreationExports {
            component: format!("{full_path}/{name}{ext}x"),
            styles: format!("{full_path}/{name}.styles{ext}"),
            responsive: format!("{full_path}/{name}.styles.responsive{ext}"),
            proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
          }
        ))
      },
      Tool::Svelte => {
        use templates::svelte::statics::component::{
          COMPONENT,
          PROPTYPES,
          SCRIPT,
          SCRIPT_TS,
          STYLES,
          STYLES_RESPONSIVE,
        };

        let proptypes = if is_ts {
          Some(CreationPaths {
            template: format!("/proptypes{ext}"),
            default: PROPTYPES.to_string(),
          })
        } else { None };
        let script = Some(CreationPaths {
          template: format!("/script{ext}.svelte"),
          default: if is_ts { SCRIPT_TS.to_string() } else { SCRIPT.to_string() }
        });

        Ok(ComponentCreation::new(
          &self.config.templates,
          format!("export {{ default as {name} }} from \"./{name}/{name}.svelte\";\n"),
          CreationPaths {
            template: format!("/styles{ext}"),
            default: STYLES.to_string(),
          },
          CreationPaths {
            template: "/component.svelte".to_string(),
            default: COMPONENT.to_string(),
          },
          CreationPaths {
            template: format!("/styles.responsive{ext}"),
            default: STYLES_RESPONSIVE.to_string(),
          },
          proptypes,
          script,
          ComponentCreationExports {
            component: format!("{full_path}/{name}.svelte"),
            styles: format!("{full_path}/{name}.styles{ext}"),
            responsive: format!("{full_path}/{name}.styles.responsive{ext}"),
            proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
          }
        ))
      },
      Tool::Vanilla => Err(anyhow!(self.error.clone())),
    };

    match self.global.generate_component(&full_path, component?, tool) {
      Ok(_) => {
        done();
        Ok(format!("{} {}", OK, style(format!("Component {name} created at {full_path}")).cyan()))
      },
      Err(error) => Err(error)
    }
  }
  pub fn make_project(&self) -> Result<String> {
    let Answers { path, .. } = &self.answers;

    self.global.generate_project()?;

    done();
    Ok(format!("{} {}", OK, style(format!("Move to {} and start a new universe", path)).cyan()))
  }
  pub fn make_page(&self) -> Result<String> {
    let Answers { name, tool, path, language, .. } = &self.answers;
    let paths = &self.config.paths;

    let i18n = true;
    let is_ts = language.as_str() == "typescript";
    let ext = if is_ts { ".ts".to_string() } else { ".js".to_string() };
    let name_capitalize = change_case(&name, None);

    let path_ui = format!("{}/{}", paths.ui, name).to_lowercase();
    let mut path_proptypes = String::new();
    let mut path_locales = &String::new();
    let mut path_i18n = String::new();

    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });

    if i18n {
      path_locales = if tool == &Tool::Svelte {
        &paths.locales.svelte
      } else {
        &paths.locales.react
      };
      path_i18n = if tool == &Tool::Svelte {
        format!("{}/i18n", paths.stores)
      } else {
        format!("{}/i18n", paths.contexts)
      };
      create_dir_all(format!("{path_locales}/en-US")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(format!("{path_locales}/es")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(path_i18n).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }
    if is_ts {
      path_proptypes = format!("{}/pages", paths.types);
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    let result = match tool {
      Tool::React => {
        use templates::react::statics::page::{
          PAGE,
          PAGE_TS,
          PROPTYPES,
          STYLES,
          STYLES_RESPONSIVE,
          ROUTER,
          ROUTE,
          LOCALE,
          I18N,
        };
        // let path_i18n_context = format!("{}/i18n", paths.contexts);

        // create_dir_all(format!("{path}/styles")).unwrap_or_else(|why| {
        //   println!("! {:?}", why.kind());
        // });
        // create_dir_all(&paths.routes).unwrap_or_else(|why| {
        //   println!("! {:?}", why.kind());
        // });
        // create_dir_all(&path_i18n_context).unwrap_or_else(|why| {
        //   println!("! {:?}", why.kind());
        // });
        // self.react.generate_page()
        // react::page::generate(
        //   path,
        //   path_proptypes.as_str(),
        //   path_locales.as_str(),
        //   paths.routes.as_str(),
        //   path_i18n_context.as_str(),
        //   &name_capitalize.as_str(),
        //   is_ts
        // )?;
        let proptypes = if is_ts {
          Some(CreationPaths {
            template: format!("/proptypes{ext}"),
            default: PROPTYPES.to_string(),
          })
        } else { None };

        let i18n_templates = if i18n {
          Some(PageCreationI18n {
            locale: CreationPaths {
              template: "/locale.json".to_string(),
              default: LOCALE.to_string()
            },
            context: CreationPaths {
              template: format!("/i18n{ext}"),
              default: I18N.to_string()
            }
          })
        } else { None };

        Ok(PageCreation::new(
          &self.config.templates,
          PageCreationImports {
            page: Some(format!("const {name} = lazy(() => (import(\"@{}/page\")));\n// ROUTES", name.to_lowercase())),
            styles: format!("export * as Page from \"./{name}.styles\";\n"),
            i18n: if i18n { Some(format!("\"{}\",\n      // NEXT_LOCALE", name.to_lowercase())) } else { None }
          },
          CreationPaths {
            template: format!("/page{ext}x"),
            default: if is_ts { PAGE_TS.to_string() } else { PAGE.to_string() }
          },
          CreationPaths {
            template: format!("/styles{ext}"),
            default: STYLES.to_string()
          },
          CreationPaths {
            template: format!("/styles.responsive{ext}"),
            default: STYLES_RESPONSIVE.to_string()
          },
          PageCreationAliases {
            config: format!("./vite.config{ext}"),
            ts_file: Some("./tsconfig.json".to_string())
          },
          Some(CreationPaths {
            template: format!("/router{ext}"),
            default: ROUTER.to_string()
          }),
          Some(CreationPaths {
            template: format!("/route{ext}"),
            default: ROUTE.to_string()
          }),
          proptypes,
          i18n_templates,
          PageCreationExports {
            
          }
        ))
      },
      Tool::Svelte => {
        // let path_i18n_store = format!("{}/i18n", paths.stores);
        // let path_ui = format!("{}/{}", paths.ui, name).to_lowercase();
        // create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
        //   println!("! {:?}", why.kind());
        // });
        // create_dir_all(&paths.pages).unwrap_or_else(|why| {
        //   println!("! {:?}", why.kind());
        // });
        // create_dir_all(&path_i18n_store).unwrap_or_else(|why| {
        //   println!("! {:?}", why.kind());
        // });
        // self.svelte.generate_page()
        // svelte::page::generate(
        //   path,
        //   path_proptypes.as_str(),
        //   path_ui.as_str(),
        //   path_locales.as_str(),
        //   path_i18n_store.as_str(),
        //   &name_capitalize.as_str(),
        //   is_ts
        // )?;
        Ok(())
      }
      Tool::Vanilla => Err(anyhow!(self.error.clone())),
    };

    match result {
      Ok(_) => {
        done();
        Ok(format!("{} {}", OK, style(format!("Page {name_capitalize} created at {path}")).cyan()))
      },
      Err(error) => Err(error)
    }
  }
  pub fn make_layout(&self) -> Result<String> {
    let Answers { name, path, tool, language, .. } = &self.answers;
    let paths = &self.config.paths;

    let name_capitalize = change_case(&name, None);
    let is_ts = language == "typescript";
    let path_typing =  format!("{}/layouts", paths.types);

    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    if is_ts {
      create_dir_all(&path_typing).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    let result = match tool {
      Tool::React => {
        create_dir_all(format!("{path}/styles")).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        self.react.generate_layout()
        // react::layout::generate(
        //   path,
        //   path_typing.as_str(),
        //   name_capitalize.as_str(),
        //   is_ts
        // )?;
      },
      Tool::Svelte => {
        let path_ui = format!("{}/{}", paths.ui, name).to_lowercase();
        create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        self.svelte.generate_layout()
        // svelte::layout::generate(
        //   path,
        //   path_typing.as_str(),
        //   path_ui.as_str(),
        //   name,
        //   is_ts
        // )?;
      }
      Tool::Vanilla => Err(anyhow!(self.error.clone())),
    };

    match result {
      Ok(_) => {
        done();
        Ok(format!( "{} {}", OK, style(format!("Layout {name_capitalize} created at {path}")).cyan()))
      },
      Err(error) => Err(error)
    }
  }
  pub fn make_schema(&self) -> Result<String> {
    let Answers { name, path, language, .. } = &self.answers;
    let paths = &self.config.paths;

    let name_dash = transform(&name, Some("dash"));
    let name_capitalize = change_case(&transform(&name, None), None);
    let name_camel = change_case(&name_capitalize, Some("camel"));
    let path_proptypes = format!("{}/schemas", paths.types);
    let namespace = *path.split('/').collect::<Vec<&str>>().last().unwrap();
    let is_ts = language == "typescript";
    
    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    if is_ts {
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    self.global.generate_schema()?;

    // global::schema::generate(
    //   &path.as_str(),
    //   path_proptypes.as_str(),
    //   &name_capitalize.as_str(),
    //   &name_dash.to_uppercase().as_str(),
    //   namespace,
    //   is_ts
    // )?;

    done();
    Ok(format!("{} {}", OK, style(format!("Schema {name_camel} created at {path}")).cyan()))
  }
  pub fn make_service(&self) -> Result<String> {
    let Answers { name, path, language, .. } = &self.answers;
    let paths = &self.config.paths;

    let name_capitalize = change_case(&name, None);
    let name_camel = change_case(&name_capitalize, Some("camel"));
    let path_proptypes = format!("{}/services", paths.types);
    let namespace = *path.split('/').collect::<Vec<&str>>().last().unwrap();

    let path_instances = path.clone().replace(namespace, "general");
    let is_ts = language == "typescript";
    
    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    create_dir_all(&path_instances).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    if is_ts {
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    self.global.generate_service()?;

    // global::service::generate(
    //   &path.as_str(),
    //   path_proptypes.as_str(),
    //   &path_instances.as_str(),
    //   &name_capitalize.as_str(),
    //   namespace,
    //   is_ts
    // )?;

    done();
    Ok(format!(
      "{} {}",
      OK,
      style(format!("Service {name_camel} created at {path}")).cyan()
    ))
  }
  pub fn make_context(&self) -> Result<String> {
    println!("make_context");
    Err(anyhow!(self.error.clone()))
  }
}