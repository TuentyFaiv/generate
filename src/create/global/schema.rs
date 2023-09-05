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
  SchemaCreation,
  SchemaCreationImports,
  SchemaCreationExports,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, language, tool, .. } = answers;
  let paths = &config.paths;

  let is_ts = *language == Lang::TypeScript;
  let ext = language.to_extension();
  let name_pascal = &name.pascal;
  let name_camel = &name.camel;
  let namespace = &name.namespace;

  let mut path_proptypes = String::new();

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });

  if is_ts {
    path_proptypes = format!("{}/schemas", paths.types);
    create_dir_all(&path_proptypes).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let schema = match tool {
    Tool::React => {
      use statics::react::schema::{
        SCHEMA,
        SCHEMA_TS,
        PROPTYPES,
      };

      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_owned(),
        })
      } else { None };

      Ok(SchemaCreation::new(
        &config.templates,
        SchemaCreationImports {
          barrel: format!("export * from \"./{name_camel}\";\n"),
          types: format!("{name_pascal}Values,\n  /* NEXT_IMPORT */")
        },
        CreationPaths {
          template: format!("/schema{ext}"),
          default: if is_ts { SCHEMA_TS.to_owned() } else { SCHEMA.to_owned() }
        },
        proptypes,
        None,
        SchemaCreationExports {
          barrel: format!("{path}/index{ext}"),
          schema: format!("{path}/{name_camel}{ext}"),
          proptypes: if is_ts { Some(format!("{path_proptypes}/{namespace}{ext}")) } else { None },
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::schema::{
        SCHEMA,
        PROPTYPES,
        PROPTYPES_IMPORTS,
      };

      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_owned(),
        })
      } else { None };

      let proptypes_imports = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes.imports{ext}"),
          default: PROPTYPES_IMPORTS.to_owned(),
        })
      } else { None };

      Ok(SchemaCreation::new(
        &config.templates,
        SchemaCreationImports {
          barrel: format!("export * from \"./{name_camel}\";\n"),
          types: format!("{name_pascal}Schema,\n  /* NEXT_IMPORT */")
        },
        CreationPaths {
          template: format!("/schema{ext}"),
          default: SCHEMA.to_owned(),
        },
        proptypes,
        proptypes_imports,
        SchemaCreationExports {
          barrel: format!("{path}/index{ext}"),
          schema: format!("{path}/{name_camel}{ext}"),
          proptypes: if is_ts { Some(format!("{path_proptypes}/{namespace}{ext}")) } else { None },
        }
      ))
    },
    Tool::Vanilla => Err(anyhow!(error.clone())),
  };

  match global.generate_schema(schema?) {
    Ok(_) => {
      done();
      Ok(format!("{} {}", OK, style(format!("Schema {name_pascal} created at {path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}