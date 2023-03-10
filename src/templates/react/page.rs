use std::io::{Write, BufReader, Read};
use std::fs::File;
use anyhow::{Result};

use super::statics::page::{
  PROPTYPES,
  PAGE_TS,
  PAGE,
  STYLES,
  STYLES_RESPONSIVE,
  ROUTER,
  ROUTE,
  LOCALE
};

pub fn generate(
  path: &str,
  path_proptypes: &str,
  path_locales: &str,
  path_routes: &str,
  name: &str,
  is_ts: bool
) -> Result<()> {

  let mut proptypes = PROPTYPES.to_string();
  let name_lower = name.to_lowercase();
  
  let mut page = PAGE.to_string();
  let mut styles = STYLES.to_string();
  let mut responsive = STYLES_RESPONSIVE.to_string();
  let mut locale = LOCALE.to_string();
  let mut router = ROUTER.to_string();
  let mut route = ROUTE.to_string();

  let ext = match is_ts {
    true => {
      page = PAGE_TS.to_string();
      ".ts".to_string()
    },
    false => {
      ".js".to_string()
    }
  };

  let page_import = format!("const {name} = lazy(() => (import(\"@{name_lower}/page\")));\n// ROUTES");

  proptypes = proptypes.replace("NAME_LOWER", &name_lower);
  page = page.replace("NAME_LOWER", &name_lower);
  styles = styles.replace("NAME_LOWER", &name_lower);
  responsive = responsive.replace("NAME_LOWER", &name_lower);
  proptypes = proptypes.replace("NAME", name);
  page = page.replace("NAME", name);
  styles = styles.replace("NAME", name);
  responsive = responsive.replace("NAME", name);
  locale = locale.replace("NAME", name);
  route = route.replace("NAME", name);
  route = route.replace("// ROUTE", format!("/{name_lower}").as_str());

  let router_path = format!("{path_routes}/router.tsx");
  let page_path = format!("{path}/+page{ext}x");
  let styles_path = format!("{path}/{name}.styles{ext}");
  let responsive_path = format!("{path}/{name}.styles.responsive{ext}");
  let locale_en_path = format!("{path_locales}/en-US/{name_lower}.json");
  let locale_es_path = format!("{path_locales}/es/{name_lower}.json");

  let mut page_file = File::create(page_path)?;
  let mut styles_file = File::create(styles_path)?;
  let mut responsive_file = File::create(responsive_path)?;
  let mut locale_en_file = File::create(locale_en_path)?;
  let mut locale_es_file = File::create(locale_es_path)?;

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
