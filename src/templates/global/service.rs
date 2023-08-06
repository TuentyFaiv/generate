use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::utils::change_case;
use crate::statics::global;

use super::CLIGlobalTemplates;

pub fn generate(CLIGlobalTemplates {
  answers,
  config,
  error
}: &CLIGlobalTemplates) -> Result<()> {
  use global::service::{PROPTYPES, SERVICE, SERVICE_TS};
  use global::service::{SERVICE_IMPORT, TYPE_EXPORT, INSTANCES};
  // For service generation

  let path = "";
  let path_proptypes = "";
  let path_instances = "";
  let name = "";
  let namespace = "";
  let is_ts = true;

  let instances = INSTANCES.to_string();
  let name_camel = change_case(name, Some("camel"));
  
  let mut proptypes = PROPTYPES.to_string();
  let mut service = SERVICE.to_string();
  let mut service_import = SERVICE_IMPORT.to_string();
  let mut type_export = TYPE_EXPORT.to_string();
  let mut ext = ".js".to_string();

  if is_ts {
    service = SERVICE_TS.to_string();
    ext = ".ts".to_string();
  }

  type_export = type_export.replace("NAME", name);
  
  service = service.replace("NAME_CAMEL", &name_camel);
  service = service.replace("NAMESPACE", namespace);
  service = service.replace("NAME", name);

  service_import = service_import.replace("NAME_CAMEL", &name_camel);

  let index_path = format!("{path}/index.ts");
  let instances_path = format!("{path_instances}/instances.ts");
  let service_path = format!("{path}/{name_camel}{ext}");
  let proptypes_path = format!("{path_proptypes}/{namespace}{ext}");

  let mut service_file = File::create(service_path)?;
  service_file.write_all(service.as_bytes())?;

  match File::open(&index_path) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), service_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index_file = File::create(&index_path)?;

      index_file.write_all(service_import.as_bytes())?;
    }
  };

  match File::open(&instances_path) {
    Ok(_) => {},
    Err(_) => {
      let mut instances_file = File::create(&instances_path)?;

      instances_file.write_all(instances.as_bytes())?;
    }
  }

  if is_ts {
    match File::open(&proptypes_path) {
      Ok(proptypes_file) => {
        let mut buf_reader = BufReader::new(&proptypes_file);
        let mut proptypes_content = String::new();
        buf_reader.read_to_string(&mut proptypes_content)?;
  
        let mut new_proptypes = File::create(&proptypes_path)?;
        
        proptypes_content = proptypes_content.replace("// NEXT_TYPE", &type_export);
  
        new_proptypes.write_all(proptypes_content.as_bytes())?;
      },
      Err(_) => {
        let mut proptypes_file = File::create(&proptypes_path)?;

        proptypes = proptypes.replace("// NEXT_TYPE", &format!("{type_export}\n"));
  
        proptypes_file.write_all(proptypes.as_bytes())?;
      }
    };
  }

  Ok(())
}