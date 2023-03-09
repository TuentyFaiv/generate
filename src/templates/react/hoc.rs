use std::io::{Write, BufReader, Read};
use std::fs::File;
use anyhow::{Result};

use super::statics::hoc::{PROPTYPES, HOC_TS, HOC};

pub fn generate(path: &str, path_proptypes: &str, name: &str, is_ts: bool) -> Result<()> {

  let proptypes = PROPTYPES.to_string();
  
  let mut hoc = HOC.to_string();
  let mut ext = ".js".to_string();
  let hoc_import = format!("export {{ with{name} }} from \"./with{name}\";\n");

  if is_ts {
    hoc = HOC_TS.to_string();
    ext = ".ts".to_string();
  }

  hoc = hoc.replace("NAME", name);

  let full_path = format!("{path}/with{name}{ext}x");
  let index_path = format!("{path}/index.ts");

  let mut hoc_file = File::create(full_path)?;
  match File::open(&index_path) {
    Ok(index) => {
      let mut buf_reader = BufReader::new(&index);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), hoc_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index = File::create(&index_path)?;

      index.write_all(hoc_import.as_bytes())?;
    }
  };
  
  hoc_file.write_all(hoc.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/with{name}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
