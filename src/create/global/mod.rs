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
            barrel: full_path.replace(format!("/{name}").as_str(), format!("/index{ext}").as_str()),
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
            barrel: full_path.replace(format!("/{name}").as_str(), format!("/index{ext}").as_str()),
            component: format!("{full_path}/{name}.svelte"),
            styles: format!("{full_path}/{name}.styles{ext}"),
            responsive: format!("{full_path}/{name}.styles.responsive{ext}"),
            proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
          }
        ))
      },
      Tool::Vanilla => Err(anyhow!(self.error.clone())),
    };

    match self.global.generate_component(component?, tool) {
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

    let path_ui = format!("{}/{}", paths.ui, name.to_lowercase());
    let mut path_proptypes = String::new();
    let mut path_locales = &String::new();
    let mut path_i18n = String::new();
    let i18n_locales = ["en-US".to_string(), "es".to_string()].to_vec();

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
      for locale in i18n_locales.clone() {
        create_dir_all(format!("{path_locales}/{locale}")).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
      }
      create_dir_all(&path_i18n).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }
    if is_ts {
      path_proptypes = format!("{}/pages", paths.types);
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    let page = match tool {
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
          VITE_CONFIG,
          VITE_ALIAS,
          TS_CONFIG,
          TS_ALIAS,
        };

        create_dir_all(&paths.routes).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });

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
            page: Some(format!("const {name_capitalize} = lazy(() => (import(\"@{}/page\")));\n// ROUTES", name.to_lowercase())),
            styles: format!("export * as Page from \"./{name}.styles\";\n"),
            i18n: if i18n { Some(format!("export * from \"./i18n/Provider{ext}\";\n")) } else { None },
            locale: if i18n { Some(format!("\"{}\",\n      // NEXT_LOCALE", name.to_lowercase())) } else { None }
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
            config: CreationPaths {
              template: format!("/config{ext}"),
              default: VITE_CONFIG.to_string(),
            },
            config_aliases: CreationPaths {
              template: format!("/config.aliases{ext}"),
              default: VITE_ALIAS.to_string(),
            },
            ts_file: if is_ts { Some(CreationPaths {
              template: format!("/tsconfig.json"),
              default: TS_CONFIG.to_string(),
            }) } else { None },
            ts_aliases: if is_ts { Some(CreationPaths {
              template: format!("/tsconfig.aliases.json"),
              default: TS_ALIAS.to_string(),
            }) } else { None },
          },
          Some(CreationPaths {
            template: format!("/router{ext}"),
            default: ROUTER.to_string()
          }),
          Some(CreationPaths {
            template: format!("/route{ext}"),
            default: ROUTE.to_string()
          }),
          None,
          proptypes,
          i18n_templates,
          PageCreationExports {
            config: format!("./vite.config{ext}"),
            page: format!("{path}/+page{ext}x"),
            barrel_styles: format!("{path_ui}/styles/index{ext}"),
            styles: format!("{path_ui}/styles/{name}.styles{ext}"),
            responsive: format!("{path_ui}/styles/{name}.styles.responsive{ext}"),
            i18n: if i18n { Some(format!("{path_i18n}/Provider{ext}")) } else { None },
            barrel_i18n: if i18n { Some(path_i18n.replace("i18n", format!("index{ext}").as_str())) } else { None },
            proptypes: if is_ts { Some(format!("{path_proptypes}/{}{ext}", name.to_lowercase())) } else { None },
            locales: Some(i18n_locales.into_iter().map(|locale| {
              format!("{path_locales}/{locale}/{}.json", name.to_lowercase())
            }).collect()),
            router: Some(format!("{}/router{ext}x", &paths.routes)),
          }
        ))
      },
      Tool::Svelte => {
        use templates::svelte::statics::page::{
          PAGE,
          SCRIPT,
          SCRIPT_TS,
          PROPTYPES,
          STYLES,
          STYLES_RESPONSIVE,
          LOCALE,
          I18N,
          SVELTE_CONFIG,
          SVELTE_ALIAS,
        };

        create_dir_all(&paths.pages).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });

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
            page: None,
            styles: format!("export * as page from \"./{}.styles\";\n", name.to_lowercase()),
            i18n: if i18n { Some(format!("export * from \"./i18n/store{ext}\";\n")) } else { None },
            locale: if i18n { Some(format!("\"{}\",\n      // NEXT_LOCALE", name.to_lowercase())) } else { None }
          },
          CreationPaths {
            template: "/page.svelte".to_string(),
            default: PAGE.to_string(),
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
            config: CreationPaths {
              template: format!("/config{ext}"),
              default: SVELTE_CONFIG.to_string(),
            },
            config_aliases: CreationPaths {
              template: format!("/config.aliases{ext}"),
              default: SVELTE_ALIAS.to_string(),
            },
            ts_file: None,
            ts_aliases: None,
          },
          None,
          None,
          Some(CreationPaths {
            template: format!("/script{ext}.svelte"),
            default: if is_ts { SCRIPT_TS.to_string() } else { SCRIPT.to_string() }
          }),
          proptypes,
          i18n_templates,
          PageCreationExports {
            config: "./svelte.config.js".to_string(),
            page: format!("{path}/+page.svelte"),
            barrel_styles: format!("{path_ui}/styles/index{ext}"),
            styles: format!("{path_ui}/styles/{}.styles{ext}", name.to_lowercase()),
            responsive: format!("{path_ui}/styles/{}.styles.responsive{ext}", name.to_lowercase()),
            i18n: if i18n { Some(format!("{path_i18n}/store{ext}")) } else { None },
            barrel_i18n: if i18n { Some(path_i18n.replace("i18n", format!("index{ext}").as_str())) } else { None },
            proptypes: if is_ts { Some(format!("{path_proptypes}/{}{ext}", name.to_lowercase())) } else { None },
            locales: Some(i18n_locales.into_iter().map(|locale| {
              format!("{path_locales}/{locale}/{}.json", name.to_lowercase())
            }).collect()),
            router: None,
          }
        ))
      }
      Tool::Vanilla => Err(anyhow!(self.error.clone())),
    };

    match self.global.generate_page(page?, tool) {
      Ok(_) => {
        done();
        Ok(format!("{} {}", OK, style(format!("Page {name_capitalize} created at {path_ui}")).cyan()))
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