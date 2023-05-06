use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::config::{Config};
use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{react, svelte};
use crate::utils::{change_case};

pub fn make(answers: &Answers, config: Config) -> Result<()> {
  let name = answers.name.as_str();
  let path = answers.path.as_str();
  let tool = answers.tool.as_str();
  let tool_type = answers.tool_type.as_str();

  let name_capitalize = change_case(name, None);
  let is_ts = tool_type == "typescript";
  let path_typing =  format!("{}/layouts", config.paths.types);

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  if is_ts {
    create_dir_all(&path_typing).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let result = match tool {
    "react" => {
      create_dir_all(format!("{path}/styles")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      react::layout::generate(
        path,
        path_typing.as_str(),
        name_capitalize.as_str(),
        is_ts
      )?;
      true
    },
    "svelte" => {
      let path_ui = format!("{}/{}", config.paths.ui, name).to_lowercase();
      create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      svelte::layout::generate(
        path,
        path_typing.as_str(),
        path_ui.as_str(),
        name,
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
      style(format!("Layout {name_capitalize} created at {path}")).cyan()
    ));
  }

  Ok(())
}
