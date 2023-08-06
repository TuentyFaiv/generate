use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::cli::enums::Tool;
use crate::create::structs::SchemaCreation;

use super::utils::{read_path, set_keywords};
use super::CLIGlobalTemplates;

pub fn generate(CLIGlobalTemplates {
  answers,
  ..
}: &CLIGlobalTemplates, templates: &SchemaCreation) -> Result<()> {
  let tool = &answers.tool;

  let template_path = match tool {
    Tool::React => templates.react_path(),
    Tool::Svelte => templates.svelte_path(),
    Tool::Vanilla => templates.vanilla_path(),
  };

  let mut schema = read_path(&template_path, &templates.schema);

  schema = set_keywords(&schema, &answers.name);

  let mut schema_file = File::create(&templates.exports.schema)?;
  schema_file.write_all(schema.as_bytes())?;

  let schema_import = &templates.import.barrel;
  let index_path = &templates.exports.barrel;
  match File::open(&index_path) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      if index_content.contains("export {};") {
        index_content = index_content.replace("export {};", "");
      }
      if index_content.contains(schema_import) {
        index_content = index_content.replace(schema_import, "");
      }

      let mut new_index = File::create(&index_path)?;
      let updated_index = [index_content.as_str(), schema_import.as_str()].concat();
      new_index.write_all(updated_index.as_bytes())?;
    },
    Err(_) => {
      let mut index_file = File::create(&index_path)?;

      index_file.write_all(schema_import.as_bytes())?;
    }
  };

  if let Some(template_props) = &templates.proptypes {
    if let Some(export_props) = &templates.exports.proptypes {
      let import_type = &templates.import.types;
      let mut proptypes = read_path(&template_path, template_props);
      let mut imports: Option<String> = None;

      if let Some(template_proptypes_imports) = &templates.proptypes_imports {
        let mut template_imports = read_path(&template_path, template_proptypes_imports);

        template_imports = set_keywords(&template_imports, &answers.name);
        template_imports = template_imports.replace("/* NEXT_IMPORT */", &import_type);
        imports = Some(template_imports);
      } 

      proptypes = set_keywords(&proptypes, &answers.name);

      match File::open(&export_props) {
        Ok(proptypes_file) => {
          let mut buf_reader = BufReader::new(&proptypes_file);
          let mut proptypes_content = String::new();
          buf_reader.read_to_string(&mut proptypes_content)?;

          let mut new_config = File::create(&export_props)?;

          proptypes_content = proptypes_content.replace("/* NEXT_IMPORT */", &import_type);
          proptypes_content = proptypes_content.replace("/* NEXT_TYPE */", &proptypes);

          new_config.write_all(proptypes_content.as_bytes())?;
        },
        Err(_) => {
          let mut proptypes_file = File::create(&export_props)?;
          
          if let Some(template_imports) = imports {
            proptypes = template_imports.replace("/* PROPTYPES */", &proptypes);
          } else {
            proptypes = proptypes.replace("/* NEXT_IMPORT */", &import_type);
          }

          proptypes_file.write_all(proptypes.as_bytes())?;
        }
      }
    }
  }
  Ok(())
}