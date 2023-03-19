use std::io::{Write, BufReader, Read};
use std::fs::File;
use anyhow::{Result};

use super::statics::page::{PROPTYPES, PAGE_TS, PAGE, STYLES, STYLES_RESPONSIVE};
use super::statics::page::{ROUTER, ROUTE, PAGE_IMPORT, STYLES_IMPORT};
use super::statics::page::{LOCALE, I18N_LOCALE, I18N};
use super::statics::page::{VITE_CONFIG, VITE_ALIAS, TS_CONFIG, TS_ALIAS};

pub fn generate(
  path: &str,
  path_proptypes: &str,
  path_locales: &str,
  path_routes: &str,
  path_i18n_context: &str,
  name: &str,
  is_ts: bool
) -> Result<()> {
  let name_lower = name.to_lowercase();
  
  let mut proptypes = PROPTYPES.to_string();
  let mut page = PAGE.to_string();
  let mut styles = STYLES.to_string();
  let mut responsive = STYLES_RESPONSIVE.to_string();

  let mut router = ROUTER.to_string();
  let mut route = ROUTE.to_string();
  let mut page_import = PAGE_IMPORT.to_string();
  let mut style_import = STYLES_IMPORT.to_string();

  let mut locale = LOCALE.to_string();
  let mut i18n = I18N.to_string();
  let mut i18n_locale = I18N_LOCALE.to_string();

  let mut ts_config = TS_CONFIG.to_string();
  let mut ts_alias = TS_ALIAS.to_string();
  let mut vite_config = VITE_CONFIG.to_string();
  let mut vite_alias = VITE_ALIAS.to_string();

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

  page_import = page_import.replace("NAME_LOWER", &name_lower);
  page_import = page_import.replace("NAME", name);
  style_import = style_import.replace("NAME", name);
  route = route.replace("NAME_LOWER", &name_lower);
  route = route.replace("NAME", name);

  i18n_locale = i18n_locale.replace("NAME_LOWER", &name_lower);
  locale = locale.replace("NAME", name);

  ts_alias = ts_alias.replace("NAME_LOWER", &name_lower);
  ts_alias = ts_alias.replace("NAME", name);
  vite_alias = vite_alias.replace("NAME_LOWER", &name_lower);
  vite_alias = vite_alias.replace("NAME", name);


  let router_path = format!("{path_routes}/router.tsx");
  let page_path = format!("{path}/+page{ext}x");
  let styles_path = format!("{path}/styles/{name}.styles{ext}");
  let responsive_path = format!("{path}/styles/{name}.styles.responsive{ext}");
  let locale_en_path = format!("{path_locales}/en-US/{name_lower}.json");
  let locale_es_path = format!("{path_locales}/es/{name_lower}.json");
  let i18n_path = format!("{path_i18n_context}/i18n{ext}");
  let styles_index = format!("{path}/styles/index{ext}");
  let tsconfig_path = "./tsconfig.json".to_string();
  let vite_path ="./vite.config.ts".to_string();

  let mut page_file = File::create(page_path)?;
  let mut styles_file = File::create(styles_path)?;
  let mut responsive_file = File::create(responsive_path)?;
  let mut locale_en_file = File::create(locale_en_path)?;
  let mut locale_es_file = File::create(locale_es_path)?;

  // Add new page to router
  match File::open(&router_path) {
    Ok(router_file) => {
      let mut buf_reader = BufReader::new(&router_file);
      let mut router_content = String::new();
      buf_reader.read_to_string(&mut router_content)?;

      let mut new_router = File::create(&router_path)?;

      router_content = router_content.replace("// ROUTES", &page_import);
      router_content = router_content.replace("// NEXT_ROUTE", &route);
      
      new_router.write_all(router_content.as_bytes())?;
    },
    Err(_) => {
      let mut router_file = File::create(&router_path)?;

      router = router.replace("// ROUTES", &page_import);
      router = router.replace("// NEXT_ROUTE", &route);

      router_file.write_all(router.as_bytes())?;
    }
  };

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

  // Set new alias to tsconfig.json
  match File::open(&tsconfig_path) {
    Ok(tsconfig_file) => {
      let mut buf_reader = BufReader::new(&tsconfig_file);
      let mut tsconfig_content = String::new();
      buf_reader.read_to_string(&mut tsconfig_content)?;

      let mut new_tsconfig = File::create(&tsconfig_path)?;

      tsconfig_content = tsconfig_content.replace("// NEXT_ALIAS", &ts_alias);

      new_tsconfig.write_all(tsconfig_content.as_bytes())?;
    },
    Err(_) => {
      let mut tsconfig_file = File::create(&tsconfig_path)?;

      ts_config = ts_config.replace("// NEXT_ALIAS", &ts_alias);

      tsconfig_file.write_all(ts_config.as_bytes())?;
    }
  }

  // Set new alias to vite.config.ts
  match File::open(&vite_path) {
    Ok(vite_file) => {
      let mut buf_reader = BufReader::new(&vite_file);
      let mut vite_content = String::new();
      buf_reader.read_to_string(&mut vite_content)?;

      let mut new_vite = File::create(&vite_path)?;

      vite_content = vite_content.replace("// NEXT_ALIAS", &vite_alias);

      new_vite.write_all(vite_content.as_bytes())?;
    },
    Err(_) => {
      let mut vite_file = File::create(&vite_path)?;

      vite_config = vite_config.replace("// NEXT_ALIAS", &vite_alias);

      vite_file.write_all(vite_config.as_bytes())?;
    }
  }

  // Set style import
  match File::open(&styles_index) {
    Ok(index_file) => {
      let mut buf_reader = BufReader::new(&index_file);
      let mut index_content = String::new();
      buf_reader.read_to_string(&mut index_content)?;

      if !index_content.contains("Page") {
        index_content = [index_content, style_import].join("");
      }

      let mut new_index = File::create(&styles_index)?;

      new_index.write_all(index_content.as_bytes())?;
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
