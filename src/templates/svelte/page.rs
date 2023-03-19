use std::io::{Write, Read, BufReader};
use std::fs::File;
use anyhow::{Result};

use super::statics::page::{PROPTYPES, PAGE_TS, PAGE};
use super::statics::page::{STYLES, STYLES_RESPONSIVE, STYLES_IMPORT};
use super::statics::page::{LOCALE, I18N_LOCALE, I18N};
use super::statics::page::{SVELTE_CONFIG, SVELTE_ALIAS};

pub fn generate(
  path: &str,
  path_proptypes: &str,
  path_ui: &str,
  path_locales: &str,
  path_i18n_store: &str,
  name: &str,
  is_ts: bool
) -> Result<()> {
  let name_lower = name.to_lowercase();
  
  let mut proptypes = PROPTYPES.to_string();
  let mut page = PAGE.to_string();
  let mut styles = STYLES.to_string();
  let mut responsive = STYLES_RESPONSIVE.to_string();

  let mut style_import = STYLES_IMPORT.to_string();

  let mut locale = LOCALE.to_string();
  let mut i18n = I18N.to_string();
  let mut i18n_locale = I18N_LOCALE.to_string();

  let mut svelte_config = SVELTE_CONFIG.to_string();
  let mut svelte_alias = SVELTE_ALIAS.to_string();

  let ext = match is_ts {
    true => {
      page = PAGE_TS.to_string();
      ".ts".to_string()
    },
    false => {
      ".js".to_string()
    }
  };

  // Set names to import and export
  proptypes = proptypes.replace("NAME_LOWER", &name_lower);
  proptypes = proptypes.replace("NAME", name);
  page = page.replace("NAME_LOWER", &name_lower);
  page = page.replace("NAME", name);

  styles = styles.replace("NAME_LOWER", &name_lower);
  styles = styles.replace("NAME", name);
  responsive = responsive.replace("NAME_LOWER", &name_lower);
  responsive = responsive.replace("NAME", name);

  style_import = style_import.replace("NAME_LOWER", &name_lower);

  i18n_locale = i18n_locale.replace("NAME_LOWER", &name_lower);
  locale = locale.replace("NAME", name);

  svelte_alias = svelte_alias.replace("NAME_LOWER", &name_lower);
  svelte_alias = svelte_alias.replace("NAME", name);


  let page_path = format!("{path}/+page.svelte");
  let styles_path = format!("{path_ui}/styles/{name_lower}.styles{ext}");
  let responsive_path = format!("{path_ui}/styles/{name_lower}.styles.responsive{ext}");
  let styles_index = format!("{path_ui}/styles/index{ext}");
  let locale_en_path = format!("{path_locales}/en-US/{name_lower}.json");
  let locale_es_path = format!("{path_locales}/es/{name_lower}.json");
  let i18n_path = format!("{path_i18n_store}/store{ext}");
  let svelte_config_path = "./svelte.config.js".to_string();

  let mut page_file = File::create(page_path)?;
  let mut styles_file = File::create(styles_path)?;
  let mut responsive_file = File::create(responsive_path)?;
  let mut locale_en_file = File::create(locale_en_path)?;
  let mut locale_es_file = File::create(locale_es_path)?;

  // Set page locale available
  match File::open(i18n_path.to_string()) {
    Ok(i18n_file) => {
      let mut buf_reader = BufReader::new(&i18n_file);
      let mut i18n_content = String::new();
      buf_reader.read_to_string(&mut i18n_content)?;

      let mut new_i18n = File::create(&i18n_path)?;

      i18n_content = i18n_content.replace("// NEXT_LOCALE", &i18n_locale);

      new_i18n.write_all(i18n_content.as_bytes())?;
    },
    Err(_) => {
      let mut i18n_file = File::create(&i18n_path)?;
      
      i18n = i18n.replace("// NEXT_LOCALE", &i18n_locale);
      
      i18n_file.write_all(i18n.as_bytes())?;
    }
  };

  // Set new alias to svelte.config.js
  match File::open(&svelte_config_path) {
    Ok(svelte_config_file) => {
      let mut buf_reader = BufReader::new(&svelte_config_file);
      let mut tsconfig_content = String::new();
      buf_reader.read_to_string(&mut tsconfig_content)?;

      let mut new_tsconfig = File::create(&svelte_config_path)?;

      tsconfig_content = tsconfig_content.replace("// NEXT_ALIAS", &svelte_alias);

      new_tsconfig.write_all(tsconfig_content.as_bytes())?;
    },
    Err(_) => {
      let mut svelte_config_file = File::create(&svelte_config_path)?;

      svelte_config = svelte_config.replace("// NEXT_ALIAS", &svelte_alias);

      svelte_config_file.write_all(svelte_config.as_bytes())?;
    }
  }

  // Set style import
  match File::open(&styles_index) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      if !index_content.contains("page") {
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

  page_file.write_all(page.as_bytes())?;
  styles_file.write_all(styles.as_bytes())?;
  responsive_file.write_all(responsive.as_bytes())?;
  locale_en_file.write_all(locale.as_bytes())?;
  locale_es_file.write_all(locale.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path_proptypes}/{name_lower}{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
