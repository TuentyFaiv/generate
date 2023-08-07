use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::config::Config;
use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{react};
use crate::utils::{change_case};

pub fn make(answers: &Answers, config: Config) -> Result<()> {
  let name = answers.name.as_str();
  let tool = answers.tool.as_str();
  let tool_type = answers.tool_type.as_str();
  let path = answers.path.as_str();

  let name_capitalize = change_case(name, None);
  let path_proptypes = format!("{}/contexts", config.paths.types);
  let is_ts = tool_type == "typescript";

  let result = match tool {
    "react" => {
      create_dir_all(path).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
      if is_ts {
        create_dir_all(&path_proptypes).unwrap_or_else(|why| {
          println!("! {:?}", why.kind());
        });
      }

      react::context::generate(
        path,
        path_proptypes.as_str(),
        &name_capitalize.as_str(),
        is_ts
      )?;
      true
    },
    _ => {false}
  };

  if result {
    done();
    msg(&format!(
      "{} {}",
      OK,
      style(format!("Context {name_capitalize} created at {path}")).cyan()
    ));
  }

  Ok(())
}
