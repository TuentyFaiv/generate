use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::{Result};

use crate::utils::{camel};

use super::statics::schema::{PROPTYPES, SCHEMA, SCHEMA_TS};
use super::statics::schema::{NEW_IMPORT, TYPE_EXPORT};

pub fn generate(
  path: &str,
  path_proptypes: &str,
  name: &str,
  name_dash: &str,
  namespace: &str,
  is_ts: bool
) -> Result<()> {
  let name_camel = camel(name);
  
  let mut proptypes = PROPTYPES.to_string();
  let mut schema = SCHEMA.to_string();
  let mut schema_import = NEW_IMPORT.to_string();
  let mut type_export = TYPE_EXPORT.to_string();
  let mut ext = ".js".to_string();

  if is_ts {
    schema = SCHEMA_TS.to_string();
    ext = ".ts".to_string();
  }

  proptypes = proptypes.replace("NAMESPACE", namespace);
  
  schema = schema.replace("NAME_DASH", name_dash);
  schema = schema.replace("NAME", name);

  schema_import = schema_import.replace("NAME_CAMEL", &name_camel);
  type_export = type_export.replace("NAME", name);

  let index_path = format!("{path}/index.ts");
  let schema_path = format!("{path}/{name_camel}{ext}");
  let proptypes_path = format!("{path_proptypes}/{namespace}{ext}");

  let mut schema_file = File::create(schema_path)?;
  schema_file.write_all(schema.as_bytes())?;

  match File::open(&index_path) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), schema_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index_file = File::create(&index_path)?;

      index_file.write_all(schema_import.as_bytes())?;
    }
  };

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
  
        proptypes = proptypes.replace("// NEXT_TYPE", &type_export);
  
        proptypes_file.write_all(proptypes.as_bytes())?;
      }
    };
  }

  Ok(())
}
