use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::{done, msg};
use crate::templates::{global};
use crate::utils::{capitalize, self};

pub fn make(
  name: &String,
  tool_type: &String,
  path: &String
) -> Result<()> {
  let name_capitalize = capitalize(&name.as_str());
  let name_camel = utils::camel(&name_capitalize.as_str());
  let path_proptypes = "./src/logic/typing/services";
  let is_ts = tool_type.as_str() == "typescript";
  
  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  if is_ts {
    create_dir_all(path_proptypes.to_string()).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  global::service::generate(
    &path.as_str(),
    path_proptypes,
    &name_capitalize.as_str(),
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
