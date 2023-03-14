use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::{done, msg};
use crate::templates::{global};
use crate::utils::{capitalize, format_dash, format_text, camel};

pub fn make(
  name: &String,
  tool_type: &String,
  path: &String
) -> Result<()> {
  let name_dash = format_dash(name);
  let name_formatted = format_text(name);
  let name_capitalize = capitalize(&name_formatted.as_str());
  let name_camel = camel(&name_capitalize.as_str());
  let path_proptypes = "./src/logic/typing/schemas";
  let path_splitted: Vec<&str> = path.split('/').collect();
  let namespace = *path_splitted.last().unwrap();
  let is_ts = tool_type.as_str() == "typescript";
  
  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  if is_ts {
    create_dir_all(path_proptypes.to_string()).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  global::schema::generate(
    &path.as_str(),
    path_proptypes,
    &name_capitalize.as_str(),
    &name_dash.to_uppercase().as_str(),
    namespace,
    is_ts
  )?;

  done();
  msg(&format!(
    "{} {}",
    OK,
    style(format!("Schema {name_camel} created at {path}")).cyan()
  ));

  Ok(())
}
