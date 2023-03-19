use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::{Result};

use super::statics::layout::{PROPTYPES, LAYOUT_TS, LAYOUT};
use super::statics::layout::{STYLES, STYLES_RESPONSIVE, STYLES_IMPORT};

pub fn generate(
  path: &str,
  path_proptypes: &str,
  name: &str,
  is_ts: bool
) -> Result<()> {
  let name_lower = name.to_lowercase();
  
  let mut proptypes = PROPTYPES.to_string();
  let mut layout = LAYOUT.to_string();
  let mut styles = STYLES.to_string();
  let mut style_import = STYLES_IMPORT.to_string();
  let mut responsive = STYLES_RESPONSIVE.to_string();

  let ext = match is_ts {
    true => {
      layout = LAYOUT_TS.to_string();
      ".ts".to_string()
    },
    false => {
      ".js".to_string()
    }
  };

  proptypes = proptypes.replace("NAME_LOWER", &name_lower);
  proptypes = proptypes.replace("NAME", name);

  layout = layout.replace("NAME_LOWER", &name_lower);
  layout = layout.replace("NAME", name);

  styles = styles.replace("NAME_LOWER", &name_lower);
  styles = styles.replace("NAME", name);

  style_import = style_import.replace("NAME", name);

  responsive = responsive.replace("NAME_LOWER", &name_lower);
  responsive = responsive.replace("NAME", name);

  let layout_path = format!("{path}/+layout{ext}x");
  let styles_path = format!("{path}/styles/{name}Layout.styles{ext}");
  let responsive_path = format!("{path}/styles/{name}Layout.styles.responsive{ext}");
  let styles_index = format!("{path}/styles/index{ext}");

  let mut layout_file = File::create(layout_path)?;
  let mut styles_file = File::create(styles_path)?;
  let mut responsive_file = File::create(responsive_path)?;

  layout_file.write_all(layout.as_bytes())?;
  styles_file.write_all(styles.as_bytes())?;
  responsive_file.write_all(responsive.as_bytes())?;


  // Set style import
  match File::open(&styles_index) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      if !index_content.contains("Layout") {
        index_content = [index_content, style_import].join("");

        let mut new_index = File::create(&styles_index)?;
  
        new_index.write_all(index_content.as_bytes())?;
      }
    },
    Err(_) => {
      let mut index_file = File::create(&styles_index)?;

      index_file.write_all(style_import.as_bytes())?;
    }
  }


  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/{name_lower}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
