use anyhow::{Result};
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::config::get_config;
use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_BASE};
use crate::statics::{ARCHS, ARCHS_REACT, ARCHS_SVELTE, ARCHS_TYPE_COMPONENT, ARCHS_VANILLA};
use crate::utils::{transform};

use super::{Args, input, choose_option, arg_or};
use super::{enums::{ArchType}};

#[derive(Debug)]
pub struct Answers {
  pub name: String,
  pub path: String,
  pub tool: String,
  pub tool_type: String,
  pub arch: String,
  pub arch_type: String,
  pub template: String,
  pub accept: bool
}

pub fn make(args: &Args) -> Result<Answers> {
  let tools = TOOLS.to_vec();
	let tools_react = TOOLS_REACT.to_vec();
	let tools_svelte = TOOLS_SVELTE.to_vec();
	let tools_webcomponents = TOOLS_WEBCOMPONENTS.to_vec();
	let tools_base = TOOLS_BASE.to_vec();

  let archs = ARCHS.to_vec();
  let archs_react = ARCHS_REACT.to_vec();
  let archs_svelte = ARCHS_SVELTE.to_vec();
  let archs_vanilla = ARCHS_VANILLA.to_vec();
	let archs_types_components = ARCHS_TYPE_COMPONENT.to_vec();

  let tool = arg_or("Choose a tool:", args.tool.clone(), tools)?;

  let mut options_archs = match tool.as_str() {
    "react" => [archs, archs_react].concat(),
    "svelte" => [archs, archs_svelte].concat(),
    "vanilla" => [archs, archs_vanilla].concat(),
    _ => archs
  };
  options_archs.sort();

  let arch = arg_or("Choose an architecture:", args.arch.clone(), options_archs)?;

  let arch_selected = match arch.as_str() {
    "atomic" => ArchType::Atomic,
    "library" => ArchType::Library,
    "component" => ArchType::Component,
    "hoc" => ArchType::Hoc,
    "hook" => ArchType::Hook,
    "context" => ArchType::Context,
    "layout" => ArchType::Layout,
    "page" => ArchType::Page,
    "service" => ArchType::Service,
    "schema" => ArchType::Schema,
    "action" => ArchType::Action,
    "store" => ArchType::Store,
    "class" => ArchType::Class,
    _ => ArchType::Atomic
  };

	let arch_type = match arch_selected {
    ArchType::Component => choose_option("Choose type:", archs_types_components)?,
    _ => String::new()
  };

	let tool_type = match format!("{tool}-{arch}").as_str() {
		"react-atomic" => {
			choose_option("Chose react project:", [tools_react, tools_base].concat())?
		},
		"svelte-atomic" => {
			choose_option("Chose svelte project:", [tools_svelte, tools_base].concat())?
		},
		"svelte-library" => {
			choose_option("Chose svelte library:", [tools_svelte, tools_webcomponents, tools_base].concat())?
		},
		_ => {
			choose_option("Chose language:", tools_base)?
		}
	};

  let mut name = match args.name.clone() {
		None => match arch_selected {
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
		Some(exist) => exist
	};

  if arch_selected == ArchType::Component
    || arch_selected == ArchType::Hoc
    || arch_selected == ArchType::Hook
    || arch_selected == ArchType::Context
    || arch_selected == ArchType::Service {
    name = transform(&name, None);
  }

  let config = get_config();

  let name_lower = name.to_lowercase();
  let namespace: &str = "sharing";
  let path_name = format!("./{name}");
	let path = match args.path.clone() {
		None => match arch_selected {
      ArchType::Hoc => config.paths.hoc,
      ArchType::Hook =>  match choose_option("Which type is:", ["global", "internal"].to_vec())?.as_str() {
        "internal" => {
          let short_path = input("Where:", namespace)?; 

          format!("{}/{}/hooks", config.paths.ui, short_path)
        },
        _ => config.paths.hook
      },
      ArchType::Page | ArchType::Layout => {
        let short_path = input("Where:", namespace)?;
        name = short_path;

        let full_path = match tool.as_str() {
          "svelte" => {
            if name.as_str() == "home" || name.as_str() == "index" {
              config.paths.page
            } else {
              format!("{}/{}", config.paths.page, name)
            }
          },
          _ => format!("{}/{}", config.paths.ui, name)
        };

        full_path
      }
      ArchType::Component => {
        let short_path = input("Where:", namespace)?;

        format!("{}/{}", config.paths.ui, short_path)
      },
      ArchType::Atomic | ArchType::Library => input("Proyect path:", &path_name.as_str())?,
      ArchType::Context => format!("{}/{}", config.paths.context, name_lower),
      ArchType::Service | ArchType::Schema => {
        let mut short_path = input("Where:", namespace)?;
        short_path = transform(&short_path, Some("lower"));
        let to = if arch_selected == ArchType::Service { config.paths.service } else { config.paths.schema };
        format!("{}/{}", to, short_path)
      },
      ArchType::Action => input("Choose location:", &config.paths.action)?,
      ArchType::Store => input("Choose location:", &config.paths.store)?,
      ArchType::Class => input("Choose location:", format!("{}/{}", config.paths.class, name_lower).as_str())?,
		}
		Some(exist) => exist
	};

	let accept = Confirm::with_theme(&ColorfulTheme::default())
		.with_prompt("Are you sure?")
		.default(true)
		.interact()?;

  let template = format!("{tool}-{tool_type}-{arch}");

  Ok(Answers {
    name,
    path,
    tool,
    tool_type,
    arch,
    arch_type,
    accept,
    template
  })
}