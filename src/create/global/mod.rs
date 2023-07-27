use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::statics::OK;
use crate::utils::{change_case, transform};
use crate::config::CLIConfig;
use crate::cli::{done, enums::Tool, structs::Answers};
use crate::templates::global::CLIGlobalTemplates;
use crate::templates::react::CLIReactTemplates;
use crate::templates::svelte::CLISvelteTemplates;

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
    let Answers { name, path, tool, tool_type, .. } = &self.answers;

    let full_path = match tool_type {
      Some(tool_type) => format!("{path}/{tool_type}/{name}"),
      None => format!("{path}/{name}")
    };

    if tool != &Tool::Vanilla {
      create_dir_all(&full_path).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    let result = match tool {
      Tool::React => {
        self.react.generate_component(&full_path)
      },
      Tool::Svelte => {
        self.svelte.generate_component(&full_path)
      },
      Tool::Vanilla => Err(anyhow!(self.error.clone())),
    };

    match result {
      Ok(_) => {
        done();
        Ok(format!("{} {}", OK, style(format!("Component {name} created at {path}")).cyan()))
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

    let is_ts = language == "typescript";
    let name_capitalize = change_case(&name, None);
    let path_proptypes = format!("{}/pages", paths.types);
    let path_locales = if tool == &Tool::Svelte {
      &paths.locales.svelte
    } else {
      &paths.locales.react
    };

    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    create_dir_all(format!("{path_locales}/en-US")).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    create_dir_all(format!("{path_locales}/es")).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    if is_ts {
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }

    let result = match tool {
      Tool::React => {
        let path_i18n_context = format!("{}/i18n", paths.contexts);

        create_dir_all(format!("{path}/styles")).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        create_dir_all(&paths.routes).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        create_dir_all(&path_i18n_context).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        self.react.generate_page()
        // react::page::generate(
        //   path,
        //   path_proptypes.as_str(),
        //   path_locales.as_str(),
        //   paths.routes.as_str(),
        //   path_i18n_context.as_str(),
        //   &name_capitalize.as_str(),
        //   is_ts
        // )?;
      },
      Tool::Svelte => {
        let path_i18n_store = format!("{}/i18n", paths.stores);
        let path_ui = format!("{}/{}", paths.ui, name).to_lowercase();
        create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        create_dir_all(&paths.pages).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        create_dir_all(&path_i18n_store).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
        self.svelte.generate_page()
        // svelte::page::generate(
        //   path,
        //   path_proptypes.as_str(),
        //   path_ui.as_str(),
        //   path_locales.as_str(),
        //   path_i18n_store.as_str(),
        //   &name_capitalize.as_str(),
        //   is_ts
        // )?;
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