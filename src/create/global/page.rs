use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::config::Config;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{react, svelte};
use crate::utils::{change_case};

pub fn make(answers: &Answers, config: Config) -> Result<()> {
  let name = answers.name.as_str();
  let tool = answers.tool.as_str();
  let tool_type = answers.tool_type.as_str();
  let path = answers.path.as_str();

  let is_ts = tool_type == "typescript";
  let name_capitalize = change_case(name, None);
  let path_proptypes = format!("{}/pages", config.paths.types);
  let path_locales = if tool == "svelte" {
    config.paths.svelte_locales
  } else {
    config.paths.react_locales
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
    "react" => {
      let path_i18n_context = format!("{}/i18n", config.paths.context);

      create_dir_all(format!("{path}/styles")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(&config.paths.routes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(&path_i18n_context).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      react::page::generate(
        path,
        path_proptypes.as_str(),
        path_locales.as_str(),
        config.paths.routes.as_str(),
        path_i18n_context.as_str(),
        &name_capitalize.as_str(),
        is_ts
      )?;

      true
    },
    "svelte" => {
      let path_i18n_store = format!("{}/i18n", config.paths.store);
      let path_ui = format!("{}/{}", config.paths.ui, name).to_lowercase();
      create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(config.paths.page).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(&path_i18n_store).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });

      svelte::page::generate(
        path,
        path_proptypes.as_str(),
        path_ui.as_str(),
        path_locales.as_str(),
        path_i18n_store.as_str(),
        &name_capitalize.as_str(),
        is_ts
      )?;

      true
    }
    "vanilla" => false,
    _ => false
  };

  if result {
    done();
    msg(&format!(
      "{} {}",
      OK,
      style(format!("Page {name_capitalize} created at {path}")).cyan()
    ));
  }

  Ok(())
}
