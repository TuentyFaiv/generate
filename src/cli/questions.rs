use anyhow::{Result};
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_BASE, ARCHS, ARCHS_COMPONENT};

use super::{choose_option, Args, input};

pub struct Answers {
  pub name: String,
  pub path: String,
  pub tool: String,
  pub tool_type: String,
  pub arch: String,
  pub arch_type: String,
  pub template: String,
  pub accept: bool,
  pub is_atomic: bool,
  pub is_library: bool,
  pub is_component: bool
}

fn format_name(name: &String) -> String {
	let splitted: Vec<&str> = name.split(&['-', ' '][..]).collect();
	
	let mut formatted = String::new();

	for word in splitted {
		let mut letters: Vec<char> = word.chars().collect();
		letters[0] = letters[0].to_uppercase().nth(0).unwrap();

		let word_capitalize: String = letters.into_iter().collect();
		
		formatted = format!("{formatted}{word_capitalize}");
	}

	formatted
}

pub fn make(args: &Args) -> Result<Answers> {
  let tools = TOOLS.to_vec();
	let tools_react = TOOLS_REACT.to_vec();
	let tools_svelte = TOOLS_SVELTE.to_vec();
	let tools_webcomponents = TOOLS_WEBCOMPONENTS.to_vec();
	let tools_base = TOOLS_BASE.to_vec();

  let archs = ARCHS.to_vec();
	let archs_components = ARCHS_COMPONENT.to_vec();

  let tool = match args.tool.clone() {
		None => {
			choose_option("Choose a tool:", tools)?
		}
		Some(exist) => {
			if !tools.contains(&exist.as_str()) {
				choose_option("Choose a tool:", tools)?
			} else {
				exist
			}
		}
	};

	let arch = match args.arch.clone() {
		None => {
			choose_option("Choose an architecture:", archs)?
		}
		Some(exist) => {
			if !archs.contains(&exist.as_str()) {
				choose_option("Choose an architecture:", archs)?
			} else {
				exist
			}
		}
	};

	let is_atomic = arch.as_str() == "atomic";
	let is_library = arch.as_str() == "library";
	let is_component = arch.as_str() == "component";

	let mut arch_type = String::new();

	if is_component {
		arch_type = choose_option("Choose type:", archs_components)?;
	}

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
		None => {
      if is_component {
        input("Component name:", "component")?
      } else {
        input("Proyect name:", "new-proyect")?
      }
		}
		Some(exist) => exist
	};

  if is_component {
    name = format_name(&name);
  }

	let path = match args.path.clone() {
		None => {
      if is_component {
        input("Component location:", format!("sharing").as_str())?
      } else {
        input("Proyect path:", format!("./{name}").as_str())?
      }
		}
		Some(exist) => exist
	};

	let accept = Confirm::with_theme(&ColorfulTheme::default())
		.with_prompt("Are you sure?")
		.default(true)
		.interact()?;

  let template = format!("{tool}-{tool_type}-{arch}");

  let answers = Answers {
    name,
    path,
    tool,
    tool_type,
    arch,
    arch_type,
    accept,
    is_atomic,
    is_library,
    is_component,
    template
  };

  Ok(answers)
}