// use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
// use console::style;

// use crate::cli::{utils::done, enums::Tool, structs::Answers};
// use crate::statics;
// use crate::statics::OK;

use super::CLIGlobalCreation;

pub fn create(CLIGlobalCreation {
  // answers,
  // config,
  error,
  // global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  Err(anyhow!(error.clone()))
  // let Answers { name, path, language, .. } = &answers;
  // let paths = &config.paths;

  // let name_pascal = &name.pascal;
  // let name_camel = &name.camel;
  // let namespace = &name.namespace;
  // let path_proptypes = format!("{}/services", paths.types);

  // let path_instances = path.clone().replace(namespace, "general");
  // let is_ts = language == "typescript";
  
  // create_dir_all(path).unwrap_or_else(|why| {
  //   println!("! {:?}", why.kind());
  // });
  // create_dir_all(&path_instances).unwrap_or_else(|why| {
  //   println!("! {:?}", why.kind());
  // });
  // if is_ts {
  //   create_dir_all(&path_proptypes).unwrap_or_else(|why| {
  //     println!("! {:?}", why.kind());
  //   });
  // }

  // global.generate_service()?;

  // // global::service::generate(
  // //   &path.as_str(),
  // //   path_proptypes.as_str(),
  // //   &path_instances.as_str(),
  // //   &name_capitalize.as_str(),
  // //   namespace,
  // //   is_ts
  // // )?;

  // done();
  // Ok(format!(
  //   "{} {}",
  //   OK,
  //   style(format!("Service {name_camel} created at {path}")).cyan()
  // ))
}