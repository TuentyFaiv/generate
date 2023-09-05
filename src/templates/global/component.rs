use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::cli::enums::Tool;
use crate::create::structs::ComponentCreation;

use super::utils::{read_path, set_keywords};
use super::CLIGlobalTemplates;

pub fn generate(CLIGlobalTemplates {
  answers,
  ..
}: &CLIGlobalTemplates, templates: &ComponentCreation) -> Result<()> {
  let tool = &answers.tool;

  let template_path = match tool {
    Tool::React => templates.react_path(),
    Tool::Svelte => templates.svelte_path(),
    Tool::Vanilla => templates.vanilla_path(),
  };

  let mut styles = read_path(&template_path, &templates.styles);
  let mut responsive = read_path(&template_path, &templates.responsive);
  let mut component = read_path(&template_path, &templates.component);
  if let Some(template_script) = &templates.script {
    let script = read_path(&template_path, template_script);
    component = component.replace("SCRIPT", &script);
  }

  component = set_keywords(&component, &answers.name);
  component = component.replace("EXT_STYLES", &templates.styles_ext);
  styles = set_keywords(&styles, &answers.name);
  responsive = set_keywords(&responsive, &answers.name);

  let mut component_file = File::create(&templates.exports.component)?;
  let mut styles_file = File::create(&templates.exports.styles)?;
  let mut responsive_file = File::create(&templates.exports.responsive)?;

  component_file.write_all(component.as_bytes())?;
  styles_file.write_all(styles.as_bytes())?;
  responsive_file.write_all(responsive.as_bytes())?;

  let component_import = &templates.import;
  let index_path = &templates.exports.barrel;
  match File::open(&index_path) {
    Ok(index) => {
      let mut buf_reader = BufReader::new(&index);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      if index_content.contains("export {};") {
        index_content = index_content.replace("export {};", "");
      }

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), component_import.as_str()].concat();

      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index = File::create(&index_path)?;

      index.write_all(component_import.as_bytes())?;
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