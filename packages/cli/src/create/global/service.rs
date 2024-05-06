use std::fs::create_dir_all;
use anyhow::Result;
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::technologies::enums::Lang;
use crate::statics::global::service::{
  INSTANCES,
  INSTANCES_TS,
  PROPTYPES,
  PROPTYPES_IMPORTS,
  SERVICE,
  SERVICE_TS
};
use crate::statics::OK;

use super::CLIGlobalCreation;
use super::structs::{
  ServiceCreation,
  ServiceCreationImports,
  ServiceCreationExports,
  CreationPaths,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  // error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, language, .. } = answers;
  let paths = &config.paths;

  let is_ts = *language == Lang::TypeScript;
  let ext = language.extension();
  let name_pascal = &name.pascal;
  let name_camel = &name.camel;
  let namespace = &name.namespace;

  let mut path_proptypes = String::new();
  let path_instances = path.replace(namespace, "globals");

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  create_dir_all(&path_instances).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });

  if is_ts {
    path_proptypes = format!("{}/services", paths.types);
    create_dir_all(&path_proptypes).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

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

  let service = ServiceCreation::new(
    &config.templates,
    ServiceCreationImports {
      barrel: format!("export * from \"./{name_camel}\";\n"),
      barrel_instances: format!("export * from \"./instances\";\n"),
    },
    CreationPaths {
      template: format!("/service{ext}"),
      default: if is_ts { SERVICE_TS.to_owned() } else { SERVICE.to_owned() }
    },
    CreationPaths {
      template: format!("/service.instances{ext}"),
      default: if is_ts { INSTANCES_TS.to_owned() } else { INSTANCES.to_owned() }
    },
    proptypes,
    proptypes_imports,
    ServiceCreationExports {
      barrel: format!("{path}/index{ext}"),
      barrel_instances: format!("{path_instances}/index{ext}"),
      service: format!("{path}/{name_camel}{ext}"),
      proptypes: if is_ts { Some(format!("{path_proptypes}/{namespace}{ext}")) } else { None },
      instances: format!("{path_instances}/instances{ext}"),
    }
  );

  match global.generate_service(service) {
    Ok(_) => {
      done();
      Ok(format!("{} {}", OK, style(format!("Service {name_pascal} created at {path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}