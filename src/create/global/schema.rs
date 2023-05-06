use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::config::Config;
use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{global};
use crate::utils::{change_case, transform};

pub fn make(answers: &Answers, config: Config) -> Result<()> {
  let name = &answers.name;
  let path = &answers.path;
  let tool_type = answers.tool_type.as_str();

  let name_dash = transform(name, Some("dash"));
  let name_capitalize = change_case(transform(name, None).as_str(), None);
  let name_camel = change_case(&name_capitalize.as_str(), Some("camel"));
  let path_proptypes = format!("{}/schemas", config.paths.types);
  let namespace = *path.split('/').collect::<Vec<&str>>().last().unwrap();
  let is_ts = tool_type == "typescript";
  
  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  if is_ts {
    create_dir_all(&path_proptypes).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  global::schema::generate(
    &path.as_str(),
    path_proptypes.as_str(),
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
