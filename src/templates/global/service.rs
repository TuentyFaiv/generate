use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::{Result};

use crate::utils;

use super::statics::service::{PROPTYPES, SERVICE, SERVICE_TS, INSTANCES};

pub fn generate(path: &str, path_proptypes: &str, name: &str, is_ts: bool) -> Result<()> {

  let mut proptypes = PROPTYPES.to_string();
  let instances = INSTANCES.to_string();
  let name_lower = name.to_lowercase();
  let name_camel = utils::camel(name);
  
  let mut service = SERVICE.to_string();
  let mut ext = ".js".to_string();
  let service_import = format!("export * from \"./{name_camel}\";\n");

  if is_ts {
    service = SERVICE_TS.to_string();
    ext = ".ts".to_string();
  }

  proptypes = proptypes.replace("NAME_CAMEL", &name_camel);
  service = service.replace("NAME_CAMEL", &name_camel);

  proptypes = proptypes.replace("NAME_LOWER", &name_lower);
  service = service.replace("NAME_LOWER", &name_lower);

  proptypes = proptypes.replace("NAME", name);
  service = service.replace("NAME", name);

  let index_path = format!("{path}/index.ts");
  let instances_path = format!("{path}/instances.ts");
  let service_path = format!("{path}/{name_camel}{ext}");

  let mut service_file = File::create(service_path)?;
  match File::open(&index_path) {
    Ok(index) => {
      let mut buf_reader = BufReader::new(&index);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), service_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index = File::create(&index_path)?;

      index.write_all(service_import.as_bytes())?;
    }
  };

  match File::open(&instances_path) {
    Ok(_) => {},
    Err(_) => {
      let mut instances_file = File::create(&instances_path)?;

      instances_file.write_all(instances.as_bytes())?;
    }
  }
  
  service_file.write_all(service.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/{name_camel}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
