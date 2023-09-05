use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::cli::enums::{Lang, Tool};
use crate::statics;
use crate::statics::OK;

use super::CLIGlobalCreation;
use super::structs::{
  CreationPaths,
  PageCreation,
  PageCreationI18n,
  PageCreationImports,
  PageCreationExports,
  PageCreationAliases,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, tool, path, language, .. } = answers;
  let paths = &config.paths;

  let i18n = true;
  let is_ts = *language == Lang::TypeScript;
  let ext = language.to_extension();
  let name_pascal = &name.pascal;

  let path_ui = format!("{}/{}", paths.ui, name.namespace);
  let mut path_proptypes = String::new();
  let mut path_locales = &String::new();
  let mut path_i18n = String::new();
  let i18n_locales = ["en-US".to_owned(), "es".to_owned()].to_vec();

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  create_dir_all(format!("{path_ui}/styles")).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });

  if i18n {
    path_locales = if tool == &Tool::Svelte {
      &paths.locales.svelte
    } else {
      &paths.locales.react
    };
    path_i18n = if tool == &Tool::Svelte {
      format!("{}/i18n", paths.stores)
    } else {
      format!("{}/i18n", paths.contexts)
    };
    for locale in i18n_locales.clone() {
      create_dir_all(format!("{path_locales}/{locale}")).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }
    create_dir_all(&path_i18n).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }
  if is_ts {
    path_proptypes = format!("{}/pages", paths.types);
    create_dir_all(&path_proptypes).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let page = match tool {
    Tool::React => {
      use statics::react::page::{
        PAGE,
        PAGE_TS,
        PROPTYPES,
        STYLES,
        STYLES_RESPONSIVE,
        ROUTER,
        ROUTE,
        LOCALE,
        I18N,
        VITE_CONFIG,
        VITE_ALIAS,
        TS_CONFIG,
        TS_ALIAS,
      };

      create_dir_all(&paths.routes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });

      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_string(),
        })
      } else { None };

      let i18n_templates = if i18n {
        Some(PageCreationI18n {
          locale: CreationPaths {
            template: "/locale.json".to_string(),
            default: LOCALE.to_string()
          },
          context: CreationPaths {
            template: format!("/i18n{ext}"),
            default: I18N.to_string()
          }
        })
      } else { None };

      Ok(PageCreation::new(
        &config.templates,
        PageCreationImports {
          page: Some(format!("const {name_pascal} = lazy(() => (import(\"@{}/page\")));\n/* NEXT_IMPORT */", name.namespace)),
          styles: format!("export * as Page from \"./{name_pascal}.styles\";\n"),
          i18n: if i18n { Some("export * from \"./i18n/Provider\";\n".to_owned()) } else { None },
          locale: if i18n { Some(format!("\"{}\",\n      /* NEXT_LOCALE */", name.namespace)) } else { None }
        },
        CreationPaths {
          template: format!("/page{ext}x"),
          default: if is_ts { PAGE_TS.to_string() } else { PAGE.to_string() }
        },
        CreationPaths {
          template: format!("/styles{ext}"),
          default: STYLES.to_string()
        },
        CreationPaths {
          template: format!("/styles.responsive{ext}"),
          default: STYLES_RESPONSIVE.to_string()
        },
        PageCreationAliases {
          config: CreationPaths {
            template: format!("/config{ext}"),
            default: VITE_CONFIG.to_string(),
          },
          config_aliases: CreationPaths {
            template: format!("/config.aliases{ext}"),
            default: VITE_ALIAS.to_string(),
          },
          ts_file: if is_ts { Some(CreationPaths {
            template: format!("/tsconfig.json"),
            default: TS_CONFIG.to_string(),
          }) } else { None },
          ts_aliases: if is_ts { Some(CreationPaths {
            template: format!("/tsconfig.aliases.json"),
            default: TS_ALIAS.to_string(),
          }) } else { None },
        },
        Some(CreationPaths {
          template: format!("/router{ext}"),
          default: ROUTER.to_string()
        }),
        Some(CreationPaths {
          template: format!("/route{ext}"),
          default: ROUTE.to_string()
        }),
        None,
        proptypes,
        i18n_templates,
        PageCreationExports {
          config: format!("./vite.config{ext}"),
          page: format!("{path}/+page{ext}x"),
          barrel_styles: format!("{path_ui}/styles/index{ext}"),
          styles: format!("{path_ui}/styles/{name_pascal}.styles{ext}"),
          responsive: format!("{path_ui}/styles/{name_pascal}.styles.responsive{ext}"),
          i18n: if i18n { Some(format!("{path_i18n}/Provider{ext}")) } else { None },
          barrel_i18n: if i18n { Some(path_i18n.replace("i18n", format!("index{ext}").as_str())) } else { None },
          proptypes: if is_ts { Some(format!("{path_proptypes}/{}{ext}", name.namespace)) } else { None },
          locales: Some(i18n_locales.into_iter().map(|locale| {
            format!("{path_locales}/{locale}/{}.json", name.namespace)
          }).collect()),
          router: Some(format!("{}/router{ext}x", &paths.routes)),
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::page::{
        PAGE,
        SCRIPT,
        SCRIPT_TS,
        PROPTYPES,
        STYLES,
        STYLES_RESPONSIVE,
        LOCALE,
        I18N,
        SVELTE_CONFIG,
        SVELTE_ALIAS,
      };

      create_dir_all(&paths.pages).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });

      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_string(),
        })
      } else { None };

      let i18n_templates = if i18n {
        Some(PageCreationI18n {
          locale: CreationPaths {
            template: "/locale.json".to_string(),
            default: LOCALE.to_string()
          },
          context: CreationPaths {
            template: format!("/i18n{ext}"),
            default: I18N.to_string()
          }
        })
      } else { None };

      Ok(PageCreation::new(
        &config.templates,
        PageCreationImports {
          page: None,
          styles: format!("export * as page from \"./{}.styles\";\n", name.namespace),
          i18n: if i18n { Some(format!("export * from \"./i18n/store\";\n")) } else { None },
          locale: if i18n { Some(format!("\"{}\",\n      /* NEXT_LOCALE */", name.namespace)) } else { None }
        },
        CreationPaths {
          template: "/page.svelte".to_string(),
          default: PAGE.to_string(),
        },
        CreationPaths {
          template: format!("/styles{ext}"),
          default: STYLES.to_string()
        },
        CreationPaths {
          template: format!("/styles.responsive{ext}"),
          default: STYLES_RESPONSIVE.to_string()
        },
        PageCreationAliases {
          config: CreationPaths {
            template: format!("/config{ext}"),
            default: SVELTE_CONFIG.to_string(),
          },
          config_aliases: CreationPaths {
            template: format!("/config.aliases{ext}"),
            default: SVELTE_ALIAS.to_string(),
          },
          ts_file: None,
          ts_aliases: None,
        },
        None,
        None,
        Some(CreationPaths {
          template: format!("/script{ext}.svelte"),
          default: if is_ts { SCRIPT_TS.to_string() } else { SCRIPT.to_string() }
        }),
        proptypes,
        i18n_templates,
        PageCreationExports {
          config: "./svelte.config.js".to_string(),
          page: format!("{path}/+page.svelte"),
          barrel_styles: format!("{path_ui}/styles/index{ext}"),
          styles: format!("{path_ui}/styles/{}.styles{ext}", name.namespace),
          responsive: format!("{path_ui}/styles/{}.styles.responsive{ext}", name.namespace),
          i18n: if i18n { Some(format!("{path_i18n}/store{ext}")) } else { None },
          barrel_i18n: if i18n { Some(path_i18n.replace("i18n", format!("index{ext}").as_str())) } else { None },
          proptypes: if is_ts { Some(format!("{path_proptypes}/{}{ext}", name.namespace)) } else { None },
          locales: Some(i18n_locales.into_iter().map(|locale| {
            format!("{path_locales}/{locale}/{}.json", name.namespace)
          }).collect()),
          router: None,
        }
      ))
    }
    Tool::Vanilla => Err(anyhow!(error.clone())),
  };

  match global.generate_page(page?) {
    Ok(_) => {
      done();
      Ok(format!("{} {}", OK, style(format!("Page {name_pascal} created at {path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}