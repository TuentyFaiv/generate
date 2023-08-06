use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::cli::enums::Tool;
use crate::create::structs::LayoutCreation;

use super::utils::{read_path, set_keywords};
use super::CLIGlobalTemplates;

pub fn generate(CLIGlobalTemplates {
  answers,
  ..
}: &CLIGlobalTemplates, templates: &LayoutCreation) -> Result<()> {
  let tool = &answers.tool;

  let template_path = match tool {
    Tool::React => templates.react_path(),
    Tool::Svelte => templates.svelte_path(),
    Tool::Vanilla => templates.vanilla_path(),
  };

  let mut styles = read_path(&template_path, &templates.styles);
  let mut responsive = read_path(&template_path, &templates.responsive);
  let mut layout = read_path(&template_path, &templates.layout);
  if let Some(template_script) = &templates.script {
    let script = read_path(&template_path, template_script);
    layout = layout.replace("SCRIPT", &script);
  }

  layout = set_keywords(&layout, &answers.name);
  styles = set_keywords(&styles, &answers.name);
  responsive = set_keywords(&responsive, &answers.name);

  let mut layout_file = File::create(&templates.exports.layout)?;
  let mut styles_file = File::create(&templates.exports.styles)?;
  let mut responsive_file = File::create(&templates.exports.responsive)?;

  layout_file.write_all(layout.as_bytes())?;
  styles_file.write_all(styles.as_bytes())?;
  responsive_file.write_all(responsive.as_bytes())?;

  let styles_import = &templates.import;
  let styles_index = &templates.exports.barrel_styles;
  match File::open(&styles_index) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      if !index_content.contains(styles_import) {
        index_content = [index_content, styles_import.to_owned()].join("");

        let mut new_index = File::create(&styles_index)?;
  
        new_index.write_all(index_content.as_bytes())?;
      }
    },
    Err(_) => {
      let mut index_file = File::create(&styles_index)?;

      index_file.write_all(styles_import.as_bytes())?;
    }
  }

  if let Some(template_props) = &templates.proptypes {
    let mut proptypes = read_path(&template_path, template_props);

    proptypes = set_keywords(&proptypes, &answers.name);

    if let Some(export_props) = &templates.exports.proptypes {
      let mut proptypes_file = File::create(export_props)?;
      proptypes_file.write_all(proptypes.as_bytes())?;
    }
  }

  Ok(())
}