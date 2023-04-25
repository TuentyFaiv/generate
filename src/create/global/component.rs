use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{react, svelte};

pub fn make(answers: &Answers) -> Result<()> {
  let name = answers.name.as_str();
  let path = answers.path.as_str();
  let tool = answers.tool.as_str();
  let tool_type = &answers.tool_type;
  let arch_type = answers.arch_type.as_str();

  let full_path = match arch_type {
    "normal" => format!("{path}/components/{name}"),
    _ => format!("{path}/{arch_type}/{name}")
  };
  
  if tool == "react" || tool == "svelte" || tool == "vanilla" {
    create_dir_all(&full_path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let result = match tool {
    "react" => {
      react::component::generate(&full_path, &name, &tool_type)?;
      true
    },
    "svelte" => {
      svelte::component::generate(&full_path, &name, &tool_type)?;
      true
    },
    "vanilla" => false,
    _ => false
  };

  if result {
    done();
    msg(&format!("{} {}", OK, style(format!("Component {name} created at {path}")).cyan()));
  }

  Ok(())
}
