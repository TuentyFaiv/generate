use anyhow::{Result};

use crate::config::{Config};
use crate::utils::{transform, join_slices, to_vec};

use super::{Args, input, choose_option, arg_or, sure};
use super::{enums::{ArchType}};

#[derive(Clone, Debug)]
pub struct Answers {
  pub path: String,
  pub tool: String,
  pub tool_type: String,
  pub arch: ArchType,
  pub arch_type: Option<String>,
  pub accept: bool,
}

pub trait Questions {
  fn ask(&self) -> Result<Answers>;
}

pub struct CLIQuestions<'a, C: Config> {
  args: Args,
  config: &'a C,
}

impl<'a, C: Config> CLIQuestions<'a, C> {
  pub fn new(config: &'a C, args: Args) -> Self {
    Self { args, config }
  }
}

impl<'a, C: Config> Questions for CLIQuestions<'a, C> {
  fn ask(&self) -> Result<Answers> {
    let paths = self.config.get_paths();
    let tools = self.config.get_tools();
    let tool = arg_or(
      "Choose a tool:",
      self.args.tool.clone(),
      &to_vec(tools.globals)
    )?;

    let archs = self.config.get_archs();
    let mut options_archs = match tool.as_str() {
      "react" => join_slices(archs.globals, archs.react),
      "svelte" => join_slices(archs.globals, archs.svelte),
      "vanilla" => join_slices(archs.globals, archs.vanilla),
      _ => to_vec(archs.globals)
    };
    options_archs.sort();

    let arch = ArchType::parse(arg_or(
      "Choose an architecture:", 
      self.args.arch.clone(),
      &options_archs
    )?.as_str());

    let arch_type = match arch {
      ArchType::Component => {
        let mut option = choose_option("Choose type:", &to_vec(archs.type_component))?.to_string();
        if option == "custom" {
          option = input("Custom type:", "component")?;
        }
        Some(option)
      },
      _ => None,
    };

    let react_tools = join_slices(tools.react, tools.vanilla);
    let mut svelte_tools = join_slices(tools.svelte, tools.vanilla);
    let vanilla_tools = to_vec(tools.vanilla);
    let tool_type = match arch {
      ArchType::Atomic => match tool.as_str() {
        "react" => choose_option("Choose react project:", &react_tools)?,
        "svelte" => choose_option("Choose svelte project:", &svelte_tools)?,
        _ => choose_option("Choose language:", &vanilla_tools)?
      },
      ArchType::Library => match tool.as_str() {
        "svelte" => {
          let formatted_svelte = svelte_tools.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
          svelte_tools = join_slices(&formatted_svelte, tools.webcomponents);
          choose_option("Choose svelte library:", &svelte_tools)?
        },
        _ => choose_option("Choose language:", &vanilla_tools)?
      },
      _ => choose_option("Choose language:", &vanilla_tools)?
    };

    let mut name = match &self.args.name {
      None => match arch {
        ArchType::Component => input("Component name:", "component")?,
        ArchType::Hoc => input("Hoc name:", "hoc")?,
        ArchType::Hook => input("Hook name:", "hook")?,
        ArchType::Context => input("Context name:", "context")?,
        ArchType::Service => input("Service name:", "service")?,
        ArchType::Schema => input("Schema name:", "schema")?,
        ArchType::Action => input("Action name:", "action")?,
        ArchType::Store => input("Store name:", "store")?,
        ArchType::Class => input("Class name:", "some-new class")?,
        ArchType::Atomic => input("Atomic name:", "atomic")?,
        ArchType::Library => input("Library name:", "library")?,
        _ => String::new()
      }
      Some(exist) => exist.clone()
    };

    if arch == ArchType::Component
      || arch == ArchType::Hoc
      || arch == ArchType::Hook
      || arch == ArchType::Context
      || arch == ArchType::Service {
      name = transform(&name, None);
    }

    let namespace: &str = "sharing";
    let name_lower = name.to_lowercase();

    let path = match &self.args.path {
      None => match arch {
        ArchType::Hoc => paths.hoc.to_string(),
        ArchType::Hook =>  match choose_option("Which type is:", &to_vec(&["global", "internal"]))?.as_str() {
          "internal" => {
            let short_path = input("Where:", namespace)?; 
            format!("{}/{}/hooks", paths.ui, short_path)
          },
          _ => paths.hook.to_string()
        },
        ArchType::Page | ArchType::Layout => {
          let short_path = input("Where:", namespace)?;
          name = short_path;

          let full_path = match tool.as_str() {
            "svelte" => {
              if name == "home" || name == "index" {
                paths.page.to_string()
              } else {
                format!("{}/{}", paths.page, name)
              }
            },
            _ => format!("{}/{}", paths.ui, name)
          };

          full_path
        }
        ArchType::Component => {
          let short_path = input("Where:", namespace)?;

          format!("{}/{}", paths.ui, short_path)
        },
        ArchType::Atomic | ArchType::Library => input("Proyect path:", &paths.get_root(&name))?,
        ArchType::Context => format!("{}/{}", paths.context, name_lower),
        ArchType::Service | ArchType::Schema => {
          let mut short_path = input("Where:", namespace)?;
          short_path = transform(&short_path, Some("lower"));
          let to = if arch == ArchType::Service { paths.service } else { paths.schema };
          format!("{}/{}", to, short_path)
        },
        ArchType::Action => input("Choose location:", paths.action)?,
        ArchType::Store => input("Choose location:", paths.store)?,
        ArchType::Class => input("Choose location:", format!("{}/{}", paths.class, name_lower).as_str())?,
      }
      Some(exist) => exist.clone()
    };

    let accept = sure()?;

    Ok(Answers {
      path,
      tool,
      tool_type,
      arch,
      arch_type,
      accept,
    })
  }
}
