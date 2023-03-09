use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::{done, msg};
use crate::templates::{react, svelte};

pub fn make(
  name: &String,
  tool: &String,
  tool_type: &String,
  arch_type: &String,
  path: &String
) -> Result<()> {
  let selected = tool.as_str();
  let full_path = match arch_type.as_str() {
    "normal" => {
      format!("{path}/components/{name}")
    },
    _ => {
      format!("{path}/{arch_type}/{name}")
    }
  };
  
  if selected == "react" || selected == "svelte" || selected == "vanilla" {
    create_dir_all(&full_path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let result = match selected {
    "react" => {
      react::component::generate(&full_path.as_str(), &name.as_str(), &tool_type)?;
      true
    },
    "svelte" => {
      svelte::component::generate(&full_path.as_str(), &name.as_str(), &tool_type)?;
      true
    },
    "vanilla" => {
      println!("Vanilla component");
      true
    },
    _ => {false}
  };

  if result {
    done();
    msg(&format!("{} {}", OK, style(format!("Component {name} created at {path}")).cyan()));
  }

  Ok(())
}
