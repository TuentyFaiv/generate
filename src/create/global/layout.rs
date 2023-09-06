use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::cli::enums::{Lang, Tool};
use crate::statics;
use crate::statics::OK;

use super::CLIGlobalCreation;
use super::structs::{
  CreationPaths,
  LayoutCreation,
  LayoutCreationExports,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, tool, language, .. } = answers;
  let paths = &config.paths;

  let is_ts = *language == Lang::TypeScript;
  let ext = language.to_extension();
  let name_pascal = &name.pascal;

  let path_ui = format!("{}/{}", paths.ui, name.namespace);
  let mut path_proptypes = String::new();

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });

  if is_ts {
    path_proptypes =  format!("{}/layouts", paths.types);
    create_dir_all(&path_proptypes).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let layout = match tool {
    Tool::React => {
      use statics::react::layout::{
        LAYOUT,
        LAYOUT_TS,
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

      Ok(LayoutCreation::new(
        &config.templates,
        format!("export * as Layout from \"./{name_pascal}Layout.styles\";\n"),
        CreationPaths {
          template: format!("/layout{ext}x"),
          default: if is_ts { LAYOUT_TS.to_string() } else { LAYOUT.to_string() }
        },
        CreationPaths {
          template: format!("/styles{ext}"),
          default: STYLES.to_string()
        },
        CreationPaths {
          template: format!("/styles.responsive{ext}"),
          default: STYLES_RESPONSIVE.to_string()
        },
        proptypes,
        None,
        LayoutCreationExports {
          barrel_styles: format!("{path_ui}/styles/index{ext}"),
          layout: format!("{path}/+layout{ext}x"),
          styles: format!("{path_ui}/styles/{name_pascal}Layout.styles{ext}"),
          responsive: format!("{path_ui}/styles/{name_pascal}Layout.styles.responsive{ext}"),
          proptypes: if is_ts { Some(format!("{path_proptypes}/{}{ext}", name.namespace)) } else { None },
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::layout::{
        LAYOUT,
        SCRIPT,
        SCRIPT_TS,
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

      Ok(LayoutCreation::new(
        &config.templates,
        format!("export * as layout from \"./{}.layout.styles\";\n", name.namespace),
        CreationPaths {
          template: "/layout.svelte".to_string(),
          default: LAYOUT.to_string()
        },
        CreationPaths {
          template: format!("/styles{ext}"),
          default: STYLES.to_string()
        },
        CreationPaths {
          template: format!("/styles.responsive{ext}"),
          default: STYLES_RESPONSIVE.to_string()
        },
        proptypes,
        Some(CreationPaths {
          template: format!("/script{ext}.svelte"),
          default: if is_ts { SCRIPT_TS.to_string() } else { SCRIPT.to_string() }
        }),
        LayoutCreationExports {
          barrel_styles: format!("{path_ui}/styles/index{ext}"),
          layout: format!("{path}/+layout.svelte"),
          styles: format!("{path_ui}/styles/{}.layout.styles{ext}", name.namespace),
          responsive: format!("{path_ui}/styles/{}.layout.styles.responsive{ext}", name.namespace),
          proptypes: if is_ts { Some(format!("{path_proptypes}/{}{ext}", name.namespace)) } else { None },
        }
      ))
    }
    Tool::Vanilla => Err(anyhow!(error.clone())),
  };

  match global.generate_layout(layout?) {
    Ok(_) => {
      done();
      Ok(format!( "{} {}", OK, style(format!("Layout {name_pascal} created at {path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}