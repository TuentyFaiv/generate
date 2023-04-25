use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{react, svelte};
use crate::utils::{change_case};

pub fn make(answers: &Answers) -> Result<()> {
  let name = answers.name.as_str();
  let tool = answers.tool.as_str();
  let tool_type = answers.tool_type.as_str();
  let path = answers.path.as_str();

  let name_capitalize = change_case(name, None);
  let is_ts = tool_type == "typescript";
  let path_proptypes = "./src/logic/typing/pages";
  let path_locales = if tool == "svelte" { "./static/locales" } else { "./public/locales"};


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
    create_dir_all(path_proptypes.to_string()).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let result = match tool {
    "react" => {
      let path_routes = "./src/logic/routes";
      let path_i18n_context = "./src/logic/contexts/i18n";
      create_dir_all(format!("{path}/styles")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(path_routes.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(path_i18n_context.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      react::page::generate(
        path,
        path_proptypes,
        path_locales,
        path_routes,
        path_i18n_context,
        &name_capitalize.as_str(),
        is_ts
      )?;
      true
    },
    "svelte" => {
      let path_routes = "./src/routes";
      let path_i18n_store = "./src/logic/stores/i18n";
      let path_ui = format!("./src/ui/{name}").to_lowercase();
      create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(path_routes.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      create_dir_all(path_i18n_store.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });

      svelte::page::generate(
        path,
        path_proptypes,
        path_ui.as_str(),
        path_locales,
        path_i18n_store,
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
