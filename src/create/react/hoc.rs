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
  let path_proptypes = "./src/logic/typing/hocs";
  let is_ts = tool_type.as_str() == "typescript";
  
  if selected == "react" {
    create_dir_all(path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
    if is_ts {
      create_dir_all(path_proptypes.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }
  }

  let result = match selected {
    "react" => {
      react::hoc::generate(
        &path.as_str(),
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
      style(format!("HOC (Higher-Order-Component) with{name_capitalize} created at {path}")).cyan()
    ));
  }

  Ok(())
}
