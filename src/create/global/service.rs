use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::config::Config;
use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{global};
use crate::utils::{change_case};

pub fn make(answers: &Answers, config: Config) -> Result<()> {
  let name = answers.name.as_str();
  let path = &answers.path;
  let tool_type = answers.tool_type.as_str();

  let name_capitalize = change_case(name, None);
  let name_camel = change_case(&name_capitalize.as_str(), Some("camel"));
  let path_proptypes = format!("{}/services", config.paths.types);
  let namespace = *path.split('/').collect::<Vec<&str>>().last().unwrap();

  let path_instances = path.clone().replace(namespace, "general");
  let is_ts = tool_type == "typescript";
  
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

  global::service::generate(
    &path.as_str(),
    path_proptypes.as_str(),
    &path_instances.as_str(),
    &name_capitalize.as_str(),
    namespace,
    is_ts
  )?;

  done();
  msg(&format!(
    "{} {}",
    OK,
    style(format!("Service {name_camel} created at {path}")).cyan()
  ));

  Ok(())
}
