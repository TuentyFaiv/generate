use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::statics::OK;
use crate::cli::{command, done, msg};

use super::svelte;

pub fn component(
  name: &String,
  tool: &String,
  tool_type: &String,
  arch_type: &String,
  path: &String
) -> Result<()> {
  let result = match tool.as_str() {
    "react" => {
      println!("React component")
    },
    "svelte" => {
      let full_path = match arch_type.as_str() {
        "normal" => {
          format!("./src/ui/{path}/components/{name}")
        },
        _ => {
          format!("./src/ui/{path}/{arch_type}/{name}")
        }
      };

      create_dir_all(&full_path).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });

      svelte::generate(&full_path.as_str(), &name.as_str(), &tool_type)?;
    },
    "vanilla" => {
      println!("Vanilla component")
    },
    _ => {}
  };

  done();
  // println!("{} {}", OK, style(format!("Component created at {path}")).cyan());
  msg(&format!("{} {}", OK, style(format!("Component {name} created at ui/{path}")).cyan()));

  Ok(result)
}

pub fn project(
  template: &Vec<&str>,
  name: &str,
  path: &str,
  tool: &str,
  arch: &str
) {
  let repository = *template.get(1).unwrap();
  let url = format!("git@github.com:Platimex/{repository}.git");
  let commit = format!("🎉 FEAT: Starting project {name}");

  command("git", ["clone", url.as_str(), path].to_vec(), None, Some(format!("Failed to generate {tool} with {arch}").as_str()));		

  // if create_library {
  // 	command("git", ["switch", "library"].to_vec(), Some(&path), Some("Failed to switch to library"));
  // }

  command("rm", ["-rf", ".git"].to_vec(), Some(path), Some("Failed to reset git"));

  command("git", ["init", "-b", "main"].to_vec(), Some(path), Some("Failed to restart git"));

  command("git", ["add", "."].to_vec(), Some(path), Some("Failed to staging files"));

  command("git", ["commit", "-m", commit.as_str(), "-m", "\"\"", "--no-gpg-sign"].to_vec(), Some(path), Some("Failed to commit"));

  command("git", ["remote", "add", "template", url.as_str()].to_vec(), Some(path), Some("Failed to add remote repository"));

  done();
  // println!("{}", style(format!("Mote to {path} and start a new universe")).cyan());
  msg(&format!("{}", style(format!("Mote to {path} and start a new universe")).cyan()));
}