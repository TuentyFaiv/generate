use anyhow::{Result};
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_BASE};
use crate::statics::{ARCHS, ARCHS_REACT, ARCHS_SVELTE, ARCHS_TYPE_COMPONENT, ARCHS_VANILLA};
use crate::utils::format_name;

use super::{choose_option, Args, input};

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

  let options_archs = match tool.as_str() {
    "react" => {
      [archs.clone(), archs_react].concat()
    },
    "svelte" => {
      [archs.clone(), archs_svelte].concat()
    },
    "vanilla" => {
      [archs.clone(), archs_vanilla].concat()
    },
    _ => {
      archs.clone()
    }
  };

	let arch = match args.arch.clone() {
		None => {
      choose_option("Choose an architecture:", options_archs)?
		}
		Some(exist) => {
			if !options_archs.contains(&exist.as_str()) {
				choose_option("Choose an architecture:", options_archs)?
			} else {
				exist
			}
		}
	};

	let is_atomic = arch.as_str() == "atomic";
	let is_library = arch.as_str() == "library";
	let is_component = arch.as_str() == "component";
	let is_hoc = arch.as_str() == "hoc";
	let is_hook = arch.as_str() == "hook";
	let is_context = arch.as_str() == "context";
	let is_layout = arch.as_str() == "layout";
	let is_page = arch.as_str() == "page";
	let is_service = arch.as_str() == "service";
	let is_schema = arch.as_str() == "schema";
	let is_action = arch.as_str() == "action";
	let is_store = arch.as_str() == "store";
	let is_class = arch.as_str() == "class";

	let mut arch_type = String::new();

	if is_component {
		arch_type = choose_option("Choose type:", archs_types_components)?;
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
      } else if is_hoc {
        input("Hoc name:", "hoc")?
      } else if is_hook {
        input("Hook name:", "hook")?
      } else if is_context {
        input("Context name:", "context")?
      } else if is_service {
        input("Service name:", "service")?
      } else if is_schema {
        input("Schema name:", "schema")?
      } else if is_action {
        input("Action name:", "action")?
      } else if is_store {
        input("Store name:", "store")?
      } else if is_class {
        input("Class name:", "some-new class")?
      } else if is_atomic {
        input("Proyect name:", "new-proyect")?
      } else {
        String::new()
      }
		}
		Some(exist) => exist
	};

  if is_component || is_hoc || is_hook {
    name = format_name(&name);
  }

	let path = match args.path.clone() {
		None => {
      let name_lower = name.to_lowercase();
      let path_name = format!("./{name}");
      let path_ui = "ui/sharing";
      let path_context = format!("./src/logic/contexts/{name_lower}");
      let path_service = "./src/logic/services";
      let path_schema = "./src/logic/schemas";
      let path_action = "./src/logic/actions";
      let path_store = "./src/logic/stores";
      let path_class = format!("./src/logic/classes/{name_lower}");

      if is_hoc {
        let path_hoc = "./src/logic/hocs";
        path_hoc.to_string()
      } else if is_hook {
        let full_path = match choose_option("Which type is:", ["global", "internal"].to_vec())?.as_str() {
          "internal" => {
            let location = input("Where:", "sharing")?;

            format!("./src/ui/{location}/hooks")
          },
          _ => {
            "./src/logic/hooks".to_string()
          }
        };

        full_path
      } else if is_component || is_page || is_layout {
        let short_path = input("Choose location:", path_ui)?;

        let full_path = format!("./src/{short_path}");

        full_path
      } else if is_atomic || is_library {
        input("Proyect path:", &path_name.as_str())?
      } else if is_context {
        input("Choose location:", &path_context.as_str())?
      } else if is_service {
        input("Choose location:", path_service)?
      } else if is_schema {
        input("Choose location:", path_schema)?
      } else if is_action {
        input("Choose location:", path_action)?
      } else if is_store {
        input("Choose location:", path_store)?
      } else if is_class {
        input("Choose location:", &path_class.as_str())?
      } else {
        input("Proyect path:", &path_name.as_str())?
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
    template
  };

  Ok(answers)
}