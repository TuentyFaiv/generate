use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::{Result};

use crate::utils;

use super::statics::schema::{PROPTYPES, SCHEMA, SCHEMA_TS};

pub fn generate(path: &str, path_proptypes: &str, name: &str, name_dash: &str, is_ts: bool) -> Result<()> {

  let mut proptypes = PROPTYPES.to_string();
  let name_lower = name.to_lowercase();
  let name_camel = utils::camel(name);
  
  let mut schema = SCHEMA.to_string();
  let mut ext = ".js".to_string();
  let schema_import = format!("export * from \"./{name_camel}\";\n");

  if is_ts {
    schema = SCHEMA_TS.to_string();
    ext = ".ts".to_string();
  }

  proptypes = proptypes.replace("NAME_DASH", name_dash);
  schema = schema.replace("NAME_DASH", name_dash);

  proptypes = proptypes.replace("NAME_CAMEL", &name_camel);
  schema = schema.replace("NAME_CAMEL", &name_camel);

  proptypes = proptypes.replace("NAME_LOWER", &name_lower);
  schema = schema.replace("NAME_LOWER", &name_lower);

  proptypes = proptypes.replace("NAME", name);
  schema = schema.replace("NAME", name);

  let index_path = format!("{path}/index.ts");
  let schema_path = format!("{path}/{name_camel}{ext}");

  let mut schema_file = File::create(schema_path)?;
  match File::open(&index_path) {
    Ok(index) => {
      let mut buf_reader = BufReader::new(&index);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), schema_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index = File::create(&index_path)?;

      index.write_all(schema_import.as_bytes())?;
    }
  };
  
  schema_file.write_all(schema.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/{name_camel}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
