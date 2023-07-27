mod statics;

use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::config::CLIConfig;
use crate::cli::structs::Answers;
use crate::utils::read_path;

pub struct CLISvelteTemplates {
  answers: Answers,
  config: CLIConfig,
}

impl CLISvelteTemplates {
  pub fn new(config: CLIConfig, answers: Answers) -> Self {
    Self { config, answers }
  }
  pub fn generate_page(&self) -> Result<()> {
    println!("generate_page");
    Ok(())
  }
  pub fn generate_layout(&self) -> Result<()> {
    println!("generate_layout");
    Ok(())
  }
  pub fn generate_component(&self, path: &String) -> Result<()> {
    use self::statics::component::{PROPTYPES, STYLES, STYLES_RESPONSIVE, COMPONENT, SCRIPT_TS, SCRIPT};
    let is_ts = self.answers.language.as_str() == "typescript";
    let name = self.answers.name.as_str();
    let ext = if is_ts { ".ts".to_string() } else { ".js".to_string() };

    let template_path = match &self.config.templates {
      Some(templates) => match &templates.svelte {
        Some(svelte) => svelte.component.clone(),
        None => None
      },
      None => None
    };

    let proptypes = read_path(
      &template_path,
      format!("/proptypes{ext}"),
      PROPTYPES.to_string()
    );
    let mut styles = read_path(
      &template_path,
      format!("/styles{ext}"),
      STYLES.to_string()
    );
    let mut responsive = read_path(
      &template_path,
      format!("/styles.responsive{ext}"),
      STYLES_RESPONSIVE.to_string()
    );
    let mut component = read_path(
      &template_path,
      format!("/component.svelte"),
      COMPONENT.to_string()
    );
    let script = read_path(
      &template_path,
      format!("/script.{ext}.svelte"),
      if is_ts { SCRIPT_TS.to_string() } else { SCRIPT.to_string() }
    );
    component = component.replace("SCRIPT", &script);
    let component_import = format!("export {{ default as {name} }} from \"./{name}/{name}.svelte\";\n");

    component = component.replace("NAME_LOWER", &name.to_lowercase());
    styles = styles.replace("NAME_LOWER", &name.to_lowercase());
    responsive = responsive.replace("NAME_LOWER", &name.to_lowercase());
    component = component.replace("NAME", name);
    styles = styles.replace("NAME", name);
    responsive = responsive.replace("NAME", name);

    let mut component_file = File::create(format!("{path}/{name}.svelte"))?;
    let mut styles_file = File::create(format!("{path}/{name}.styles{ext}"))?;
    let mut responsive_file = File::create(format!("{path}/{name}.styles.responsive{ext}"))?;

    if is_ts {
      let mut proptypes_file = File::create(format!("{path}/{name}.proptypes{ext}"))?;
      proptypes_file.write_all(proptypes.as_bytes())?;
    }
    
    component_file.write_all(component.as_bytes())?;
    styles_file.write_all(styles.as_bytes())?;
    responsive_file.write_all(responsive.as_bytes())?;

    let mut index_path = path.replace(format!("/{name}").as_str(), "");
    index_path = format!("{index_path}/index.ts");

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

        index.write_all(component_import.as_bytes())?
      }
    }

    Ok(())
  }
}