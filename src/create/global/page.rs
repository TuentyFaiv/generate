use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::{done, msg};
use crate::templates::{react};
use crate::utils::capitalize;

pub fn make(
  name: &String,
  tool: &String,
  tool_type: &String,
  path: &String
) -> Result<()> {
  let selected = tool.as_str();
  let name_capitalize = capitalize(&name.as_str());
  let is_ts = tool_type.as_str() == "typescript";
  let path_proptypes = "./src/logic/typing/pages";
  let path_locales = "./public/locales";

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

  let result = match selected {
    "react" => {
      let path_routes = "./src/logic/routes";
      create_dir_all(path_routes.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });  
      react::page::generate(
        &path.as_str(),
        path_proptypes,
        path_locales,
        path_routes,
        &name_capitalize.as_str(),
        is_ts
      )?;
      true
    },
    "svelte" => {false}
    "vanilla" => {false}
    _ => {false}
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
