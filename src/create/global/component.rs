use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::cli::{utils::done, enums::Tool, structs::Answers};
use crate::statics;
use crate::statics::OK;

use super::CLIGlobalCreation;
use super::structs::{
  CreationPaths,
  ComponentCreation,
  ComponentCreationExports,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, tool, tool_type, language, .. } = answers;
  let is_ts = language.as_str() == "typescript";
  let ext = if is_ts { ".ts".to_owned() } else { ".js".to_owned() };
  let name = &name.pascal;

  let full_path = match tool_type {
    Some(tool_type) => format!("{path}/{tool_type}/{name}"),
    None => format!("{path}/{name}")
  };

  if tool != &Tool::Vanilla {
    create_dir_all(&full_path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let component = match tool {
    Tool::React => {
      use statics::react::component::{
        COMPONENT,
        COMPONENT_TS,
        PROPTYPES,
        STYLES,
        STYLES_RESPONSIVE,
      };
      
      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_string(),
        })
      } else { None };

      Ok(ComponentCreation::new(
        &config.templates,
        format!("export {{ default as {name} }} from \"./{name}/{name}\";\n"),
        CreationPaths {
          template: format!("/styles{ext}"),
          default: STYLES.to_string(),
        },
        CreationPaths {
          template: format!("/component{ext}x"),
          default: if is_ts { COMPONENT_TS.to_string() } else { COMPONENT.to_string() },
        },
        CreationPaths {
          template: format!("/styles.responsive{ext}"),
          default: STYLES_RESPONSIVE.to_string(),
        },
        proptypes,
        None,
        ComponentCreationExports {
          barrel: full_path.replace(format!("/{name}").as_str(), format!("/index{ext}").as_str()),
          component: format!("{full_path}/{name}{ext}x"),
          styles: format!("{full_path}/{name}.styles{ext}"),
          responsive: format!("{full_path}/{name}.styles.responsive{ext}"),
          proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::component::{
        COMPONENT,
        PROPTYPES,
        SCRIPT,
        SCRIPT_TS,
        STYLES,
        STYLES_RESPONSIVE,
      };

      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_string(),
        })
      } else { None };
      let script = Some(CreationPaths {
        template: format!("/script{ext}.svelte"),
        default: if is_ts { SCRIPT_TS.to_string() } else { SCRIPT.to_string() }
      });

      Ok(ComponentCreation::new(
        &config.templates,
        format!("export {{ default as {name} }} from \"./{name}/{name}.svelte\";\n"),
        CreationPaths {
          template: format!("/styles{ext}"),
          default: STYLES.to_string(),
        },
        CreationPaths {
          template: "/component.svelte".to_string(),
          default: COMPONENT.to_string(),
        },
        CreationPaths {
          template: format!("/styles.responsive{ext}"),
          default: STYLES_RESPONSIVE.to_string(),
        },
        proptypes,
        script,
        ComponentCreationExports {
          barrel: full_path.replace(format!("/{name}").as_str(), format!("/index{ext}").as_str()),
          component: format!("{full_path}/{name}.svelte"),
          styles: format!("{full_path}/{name}.styles{ext}"),
          responsive: format!("{full_path}/{name}.styles.responsive{ext}"),
          proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
        }
      ))
    },
    Tool::Vanilla => Err(anyhow!(error.clone())),
  };

  match global.generate_component(component?) {
    Ok(_) => {
      done();
      Ok(format!("{} {}", OK, style(format!("Component {name} created at {full_path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}