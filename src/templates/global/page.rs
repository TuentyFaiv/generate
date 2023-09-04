use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::Result;

use crate::cli::enums::Tool;
use crate::create::structs::PageCreation;

use super::utils::{read_path, set_keywords};
use super::CLIGlobalTemplates;

pub fn generate(CLIGlobalTemplates {
  answers,
  ..
}: &CLIGlobalTemplates, templates: &PageCreation) -> Result<()> {
  let tool = &answers.tool;

  let template_path = match tool {
    Tool::React => templates.react_path(),
    Tool::Svelte => templates.svelte_path(),
    Tool::Vanilla => templates.vanilla_path(),
  };

  let mut config = read_path(&template_path, &templates.aliases.config);
  let mut aliases = read_path(&template_path, &templates.aliases.config_aliases);
  let mut styles = read_path(&template_path, &templates.styles);
  let mut responsive = read_path(&template_path, &templates.responsive);
  let mut page = read_path(&template_path, &templates.page);
  if let Some(template_script) = &templates.script {
    let script = read_path(&template_path, template_script);
    page = page.replace("SCRIPT", &script);
  }

  page = set_keywords(&page, &answers.name);
  styles = set_keywords(&styles, &answers.name);
  responsive = set_keywords(&responsive, &answers.name);
  aliases = set_keywords(&aliases, &answers.name);

  let mut page_file = File::create(&templates.exports.page)?;
  let mut styles_file = File::create(&templates.exports.styles)?;
  let mut responsive_file = File::create(&templates.exports.responsive)?;

  page_file.write_all(page.as_bytes())?;
  styles_file.write_all(styles.as_bytes())?;
  responsive_file.write_all(responsive.as_bytes())?;

  // Project config file
  let config_export = &templates.exports.config;
  match File::open(&config_export) {
    Ok(config_file) => {
      let mut buf_reader = BufReader::new(&config_file);
      let mut config_content = String::new();
      buf_reader.read_to_string(&mut config_content)?;

      if !config_content.contains(&aliases) {
        config_content = config_content.replace("/* NEXT_ALIAS */", &aliases);
      }

      let mut new_config = File::create(&config_export)?;
      new_config.write_all(config_content.as_bytes())?;
    },
    Err(_) => {
      if !config.contains(&aliases) {
        config = config.replace("/* NEXT_ALIAS */", &aliases);
      }

      let mut config_file = File::create(&config_export)?;
      config_file.write_all(config.as_bytes())?;
    }
  }

  // Styles barrel file
  let styles_import = &templates.imports.styles;
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

  // Router for only React projects
  if let Some(templates_router) = &templates.router {
    if let Some(templates_route) = &templates.route {
      let mut router = read_path(&template_path, templates_router);
      let mut route = read_path(&template_path, templates_route);

      route = set_keywords(&route, &answers.name);

      if let Some(page_import) = &templates.imports.page {
        if let Some(router_export) = &templates.exports.router {
          match File::open(&router_export) {
            Ok(router_file) => {
              let mut buf_reader = BufReader::new(&router_file);
              let mut router_content = String::new();
              buf_reader.read_to_string(&mut router_content)?;

              if !router_content.contains(page_import) {
                router_content = router_content.replace("/* NEXT_IMPORT */", &page_import);
              }
              if !router_content.contains(&route) {
                router_content = router_content.replace("/* NEXT_ROUTE */", &route);
              }

              let mut new_router = File::create(&router_export)?;
              new_router.write_all(router_content.as_bytes())?;
            },
            Err(_) => {
              if !router.contains(page_import) {
                router = router.replace("/* NEXT_IMPORT */", &page_import);
              }
              if !router.contains(&route) {
                router = router.replace("/* NEXT_ROUTE */", &route);
              }

              let mut router_file = File::create(&router_export)?;
              router_file.write_all(router.as_bytes())?;
            }
          };
        }
      }
    }
  }

  // I18n config
  if let Some(templates_i18n) = &templates.i18n {
    let mut i18n_context = read_path(&template_path, &templates_i18n.context);
    let mut i18n_locale = read_path(&template_path, &templates_i18n.locale);

    i18n_locale = set_keywords(&i18n_locale, &answers.name);

    if let Some(locale_import) = &templates.imports.locale {
      if let Some(i18n_export) = &templates.exports.i18n {
        match File::open(i18n_export.to_string()) {
          Ok(i18n_file) => {
            let mut buf_reader = BufReader::new(&i18n_file);
            let mut i18n_content = String::new();
            buf_reader.read_to_string(&mut i18n_content)?;

            if !i18n_content.contains(locale_import) {
              i18n_content = i18n_content.replace("/* NEXT_LOCALE */", &locale_import);

              let mut new_i18n = File::create(&i18n_export)?;  
              new_i18n.write_all(i18n_content.as_bytes())?;
            }
          },
          Err(_) => {
            if !i18n_context.contains(locale_import) {
              i18n_context = i18n_context.replace("/* NEXT_LOCALE */", &locale_import);
            }

            let mut i18n_file = File::create(&i18n_export)?;
            i18n_file.write_all(i18n_context.as_bytes())?;
          }
        };

        if let Some(i18n_barrel) = &templates.exports.barrel_i18n {
          if let Some(i18n_import) = &templates.imports.i18n {
            match File::open(&i18n_barrel) {
              Ok(index) => {
                let mut buf_reader = BufReader::new(&index);
                let mut index_content = String::new();
                buf_reader.read_to_string(&mut index_content)?;
  
                if index_content.contains("export {};") {
                  index_content = index_content.replace("export {};", "");
                }
                if !index_content.contains(i18n_import) {
                  let updated_index = [index_content.as_str(), i18n_import.as_str()].concat();

                  let mut new_index = File::create(&i18n_barrel)?;
                  new_index.write_all(updated_index.as_bytes())?;
                }
              },
              Err(_) => {
                let mut index = File::create(&i18n_barrel)?;
                index.write_all(i18n_import.as_bytes())?;
              }
            }
          }
        }

        if let Some(i18n_langs) = &templates.exports.locales {
          for lang in i18n_langs {
            match File::open(&lang) {
              Ok(_) => {},
              Err(_) => {
                let mut file_lang = File::create(lang)?;
                file_lang.write_all(i18n_locale.as_bytes())?;
              }
            }
          }
        }
      }
    }
  }

  // Proptypes file
  if let Some(template_props) = &templates.proptypes {
    let mut proptypes = read_path(&template_path, template_props);

    proptypes = set_keywords(&proptypes, &answers.name);

    if let Some(export_props) = &templates.exports.proptypes {
      match File::open(&export_props) {
        Ok(_) => {},
        Err(_) => {
          let mut proptypes_file = File::create(export_props)?;
          proptypes_file.write_all(proptypes.as_bytes())?;
        }
      }
    }
  }

  // Typescript config (only for React projects)
  if let Some(template_tsconfig) = &templates.aliases.ts_file {
    if let Some(template_tsaliases) = &templates.aliases.ts_aliases {
      let mut tsconfig = read_path(&template_path, template_tsconfig);

      let mut ts_aliases = read_path(&template_path, template_tsaliases);

      ts_aliases = set_keywords(&ts_aliases, &answers.name);

      let tsconfig_export = "./tsconfig.json".to_string();

      match File::open(&tsconfig_export) {
        Ok(tsconfig_file) => {
          let mut buf_reader = BufReader::new(&tsconfig_file);
          let mut tsconfig_content = String::new();
          buf_reader.read_to_string(&mut tsconfig_content)?;

          let mut new_tsconfig = File::create(&tsconfig_export)?;

          if !tsconfig_content.contains(&ts_aliases) {
            tsconfig_content = tsconfig_content.replace("/* NEXT_ALIAS */", &ts_aliases);
          }

          new_tsconfig.write_all(tsconfig_content.as_bytes())?;
        },
        Err(_) => {
          let mut tsconfig_file = File::create(&tsconfig_export)?;

          if !tsconfig.contains(&ts_aliases) {
            tsconfig = tsconfig.replace("/* NEXT_ALIAS */", &ts_aliases);
          }

          tsconfig_file.write_all(tsconfig.as_bytes())?;
        }
      }
    }
  }

  Ok(())
}