use std::io::{Write, BufReader, Read};
use std::fs::File;
use anyhow::{Result};

use super::statics::hook::{PROPTYPES, HOOK_TS, HOOK};

pub fn generate(path: &str, path_proptypes: &str, name: &str, is_ts: bool) -> Result<()> {

  let mut proptypes = PROPTYPES.to_string();
  
  let mut hook = HOOK.to_string();
  let mut ext = ".js".to_string();
  let hook_import = format!("export {{ use{name} }} from \"./use{name}\";\n");

  if is_ts {
    hook = HOOK_TS.to_string();
    ext = ".ts".to_string();
  }

  hook = hook.replace("NAME", name);
  proptypes = proptypes.replace("NAME", name);

  let full_path = format!("{path}/use{name}{ext}");
  let index_path = format!("{path}/index.ts");

  let mut hook_file = File::create(full_path)?;
  match File::open(&index_path) {
    Ok(index) => {
      let mut buf_reader = BufReader::new(&index);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), hook_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index = File::create(&index_path)?;

      index.write_all(hook_import.as_bytes())?;
    }
  };
  
  hook_file.write_all(hook.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/use{name}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
