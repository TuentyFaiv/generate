use std::fs::{File, create_dir_all};
use std::io::{Write, BufWriter};
use anyhow::{Result, anyhow};
use dirs_next::home_dir;

use crate::cli::enums::{Lang, Styles};
use crate::cli::utils::{input, sure};

use super::file::ConfigFile;
use super::structs::ConfigStored;

pub fn build_global_config(config_file: &Option<ConfigFile>, set_global: bool, path: &str) -> Result<Option<ConfigStored>> {
  match config_file {
    Some(file) => {
      let storaged_i18n = match &config_file {
        Some(file) => match file.i18n {
          Some(i18n) => if set_global { i18n_question()? } else { i18n },
          None => i18n_question()?,
        },
        None => i18n_question()?,
      };
      let storaged_lang = match &config_file {
        Some(file) => match &file.lang {
          Some(lang) => if set_global { lang_question()? } else { Lang::parse(&lang) },
          None => lang_question()?,
        },
        None => lang_question()?,
      };
      let storaged_styles = match &config_file {
        Some(file) => match &file.styles {
          Some(styles) => if set_global { styles_question()? } else { Styles::parse(&styles) },
          None => styles_question()?,
        },
        None => styles_question()?,
      };

      if set_global {
        let global_config = ConfigFile {
          i18n: Some(storaged_i18n),
          lang: Some(storaged_lang.to_string()),
          styles: Some(storaged_styles.to_string()),
          paths: file.paths.clone(),
          repository: file.repository.clone(),
          root: file.root.clone(),
          templates: file.templates.clone(),
          tools_type: file.tools_type.clone(),
        };
  
        let _ = match home_dir() {
          Some(mut full_path) => {
            full_path.push(path);
  
            let dir = full_path.to_str()
              .unwrap_or(path)
              .to_string().replace("/config_cli.json", "");
  
            create_dir_all(dir).unwrap_or_else(|why| {
              println!("! {:?}", why.kind());
            });
  
            let file = File::create(&full_path)?;
            let mut buf_writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut buf_writer, &global_config)?;
            buf_writer.flush()?;
  
            Ok(())
          },
          None => Err(anyhow!("Error to find HOME path")),
        };
      }

      Ok(Some(ConfigStored { i18n: storaged_i18n, lang: storaged_lang, styles: storaged_styles }))
    },
    None => Ok(None),
  }
}

fn i18n_question() -> Result<bool> {
  sure("Do you want to use i18n? (internationalization)")
}

fn lang_question() -> Result<Lang> {
  Ok(Lang::parse(&input("Do you want to use TypeScript or JavaScript?", "typescript")?))
}

fn styles_question() -> Result<Styles> {
  Ok(Styles::parse(&input("What kind of styles do you want to use?", "emotion")?))
}