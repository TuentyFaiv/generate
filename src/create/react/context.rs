use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{react};
use crate::utils::capitalize;

pub fn make(answers: &Answers) -> Result<()> {
  let name = answers.name.as_str();
  let tool = answers.tool.as_str();
  let tool_type = answers.tool_type.as_str();
  let path = answers.path.as_str();

  let name_capitalize = capitalize(name);
  let path_proptypes = "./src/logic/typing/contexts";
  let is_ts = tool_type == "typescript";
  
  if tool == "react" {
    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    if is_ts {
      create_dir_all(path_proptypes.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }
  }

  let result = match tool {
    "react" => {
      react::context::generate(
        path,
        path_proptypes,
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
