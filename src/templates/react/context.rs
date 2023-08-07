use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use super::statics::context::{PROPTYPES, CONTEXT, CONTEXT_TS, REDUCER_TS, REDUCER};

pub fn generate(path: &str, path_proptypes: &str, name: &str, is_ts: bool) -> Result<()> {

  let mut proptypes = PROPTYPES.to_string();
  let name_lower = name.to_lowercase();
  
  let mut context = CONTEXT.to_string();
  let mut reducer = REDUCER.to_string();
  let mut ext = ".js".to_string();
  let context_import = format!("export * from \"./{name_lower}/{name}Provider\";\n");

  if is_ts {
    context = CONTEXT_TS.to_string();
    reducer = REDUCER_TS.to_string();
    ext = ".ts".to_string();
  }

  proptypes = proptypes.replace("NAME_LOWER", &name_lower);
  proptypes = proptypes.replace("NAME", name);

  context = context.replace("NAME_LOWER", &name_lower);
  context = context.replace("NAME", name);

  reducer = reducer.replace("NAME_LOWER", &name_lower);
  reducer = reducer.replace("NAME", name);

  let mut index_path = path.replace(format!("/{name_lower}").as_str(), "");
  let context_path = format!("{path}/{name}Provider{ext}x");
  let reducer_path = format!("{path}/reducer{ext}");
  index_path = format!("{index_path}/index.ts");

  let mut context_file = File::create(context_path)?;
  let mut reducer_file = File::create(reducer_path)?;
  match File::open(&index_path) {
    Ok(index) => {
      let mut buf_reader = BufReader::new(&index);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), context_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index = File::create(&index_path)?;

      index.write_all(context_import.as_bytes())?;
    }
  };
  
  context_file.write_all(context.as_bytes())?;
  reducer_file.write_all(reducer.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/{name_lower}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
