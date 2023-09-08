use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::cli::enums::{Lang, Tool, Styles};
use crate::statics;
use crate::statics::OK;
use crate::templates::constants::{
  NAMESPACE,
  NAME_PASCAL
};

use statics::global::styles::{
  STYLES_CSS,
  STYLES_CSS_RESPONSIVE,
  STYLES_LESS,
  STYLES_LESS_RESPONSIVE,
  STYLES_POSTCSS,
  STYLES_POSTCSS_RESPONSIVE,
  STYLES_SASS,
  STYLES_SASS_RESPONSIVE,
  STYLES_STYLUS,
  STYLES_STYLUS_RESPONSIVE
};
use statics::global::i18n::LOCALE_IMPORT;

use super::CLIGlobalCreation;
use super::structs::{
  CreationPaths,
  PageCreation,
  PageCreationI18n,
  PageCreationImports,
  PageCreationExports,
  PageCreationAliases,
};

use super::constants::{
  INDEX_PATH,
  I18N_PATH,
  I18N_REACT_FILE,
  I18N_SVELTE_FILE,
  I18N_CONTEXT,
  LOCALE_FILE,
  LOCALE_IMPORT_PATH,
  PROPTYPES_PATH,
  STYLES_PATH,
  STYLES_EXT,
  RESPONSIVE_PATH,
  RESPONSIVE_EXT,
  PAGE_PATH,
  PAGE_PROPS_PATH,
  PAGE_ROUTER_PATH,
  PAGE_ROTUE_PATH,
  PAGE_FILE,
  PAGE_BARREL,
  PAGE_IMPORT_PATH,
  SVELTE_EXT,
  SCRIPT_PATH,
  JSON_EXT,
  TS_CONFIG_PATH,
  TS_ALIASES_PATH,
  VITE_CONFIG_PATH,
  PAGE_CONFIG_PATH,
  PAGE_ALIASES_PATH,
  SVELTE_CONFIG_PATH,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, tool, path, language, styles, .. } = answers;
  let paths = &config.paths;

  let i18n = true;
  let styles_ext = styles.to_extension(language);
  let ext = language.to_extension();
  let name_pascal = &name.pascal;

  let path_ui = format!("{}/{}", paths.ui, name.namespace);
  let mut path_proptypes = String::new();
  let mut path_locales = &String::new();
  let mut path_i18n = String::new();
  let i18n_locales = ["en-US".to_owned(), "es".to_owned()].to_vec();

  let styles_name = match tool {
    Tool::React => name_pascal,
    Tool::Svelte => &name.namespace,
    Tool::Vanilla => name_pascal,
  };

  let styles_export = match styles {
    Styles::Emotion | Styles::StyledComponents => format!("{path_ui}{STYLES_PATH}/{styles_name}{STYLES_EXT}{styles_ext}"),
    _ => format!("{path_ui}{STYLES_PATH}/{styles_name}{styles_ext}"),
  };

  let responsive_export = match styles {
    Styles::Emotion | Styles::StyledComponents => format!("{path_ui}{STYLES_PATH}/{styles_name}{STYLES_EXT}{RESPONSIVE_EXT}{styles_ext}"),
    _ => format!("{path_ui}{STYLES_PATH}/{styles_name}{RESPONSIVE_EXT}{styles_ext}"),
  };

  let proptypes_export = match language {
    Lang::TypeScript => Some(format!("{path_proptypes}/{}{ext}", name.namespace)),
    Lang::JavaScript => None,
  };

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  create_dir_all(format!("{path_ui}{STYLES_PATH}")).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });

  let i18n_locale_import = match i18n {
    true => Some(CreationPaths {
      template: LOCALE_IMPORT_PATH.to_owned(),
      default: LOCALE_IMPORT.to_owned(),
    }),
    false => None,
  };

  if i18n {
    path_locales = match tool {
      Tool::React => &paths.locales.react,
      Tool::Svelte => &paths.locales.svelte,
      Tool::Vanilla => &paths.locales.react,
    };
    path_i18n = match tool {
      Tool::React => format!("{}{I18N_PATH}", paths.contexts),
      Tool::Svelte => format!("{}{I18N_PATH}", paths.stores),
      Tool::Vanilla => format!("{}{I18N_PATH}", paths.contexts),
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

  match language {
    Lang::TypeScript => {
      path_proptypes = format!("{}{PAGE_PROPS_PATH}", paths.types);
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    }
    Lang::JavaScript => {}
  };

  let page = match tool {
    Tool::React => {
      use statics::react::page::{
        PAGE_CSS,
        PAGE_TS_CSS,
        PAGE_STYLED,
        PAGE_TS_STYLED,
        PAGE_IMPORT,
        BARREL_STYLES_CSS,
        BARREL_STYLES_STYLED,
        I18N_IMPORT,
        PROPTYPES,
        STYLES_EMOTION,
        STYLES_EMOTION_RESPONSIVE,
        STYLES_STYLED,
        STYLES_STYLED_RESPONSIVE,
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

      let i18n_templates = match i18n {
        true => Some(PageCreationI18n {
          locale: CreationPaths {
            template: LOCALE_FILE.to_owned(),
            default: LOCALE.to_owned()
          },
          context: CreationPaths {
            template: format!("{I18N_PATH}{ext}"),
            default: I18N.to_owned()
          }
        }),
        false => None,
      };

      let i18n_barrel = match i18n {
        true => Some(path_i18n.replace("i18n", format!("index{ext}").as_str())),
        false => None,
      };

      let i18n_context = match i18n {
        true => Some(format!("{path_i18n}{I18N_REACT_FILE}{ext}")),
        false => None,
      };

      let i18n_import = match i18n {
        true => Some(CreationPaths {
          template: format!("{I18N_CONTEXT}{ext}"),
          default: I18N_IMPORT.to_owned()
        }),
        false => None,
      };

      let page_template = match styles {
        Styles::Emotion | Styles::StyledComponents => match language {
          Lang::TypeScript => PAGE_TS_STYLED.to_owned(),
          Lang::JavaScript => PAGE_STYLED.to_owned(),
        },
        _ => match language {
          Lang::TypeScript => PAGE_TS_CSS.to_owned(),
          Lang::JavaScript => PAGE_CSS.to_owned(),
        },
      };

      let styles_import = match styles {
        Styles::Emotion | Styles::StyledComponents => BARREL_STYLES_STYLED.to_owned(),
        _ => BARREL_STYLES_CSS.to_owned(),
      };

      let styles_template = match styles {
        Styles::Emotion => STYLES_EMOTION.to_owned(),
        Styles::StyledComponents => STYLES_STYLED.to_owned(),
        Styles::CSS => STYLES_CSS.to_owned(),
        Styles::LESS => STYLES_LESS.to_owned(),
        Styles::SCSS => STYLES_SASS.to_owned(),
        Styles::Stylus => STYLES_STYLUS.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS.to_owned(),
      };

      let responsive_template = match styles {
        Styles::Emotion => STYLES_EMOTION_RESPONSIVE.to_owned(),
        Styles::StyledComponents => STYLES_STYLED_RESPONSIVE.to_owned(),
        Styles::CSS => STYLES_CSS_RESPONSIVE.to_owned(),
        Styles::LESS => STYLES_LESS_RESPONSIVE.to_owned(),
        Styles::SCSS => STYLES_SASS_RESPONSIVE.to_owned(),
        Styles::Stylus => STYLES_STYLUS_RESPONSIVE.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS_RESPONSIVE.to_owned(),
      };

      let proptypes = match language {
        Lang::TypeScript => Some(CreationPaths {
          template: format!("{PROPTYPES_PATH}{ext}"),
          default: PROPTYPES.to_owned(),
        }),
        Lang::JavaScript => None,
      };

      let ts_file = match language {
        Lang::TypeScript => Some(CreationPaths {
          template: TS_CONFIG_PATH.to_owned(),
          default: TS_CONFIG.to_owned(),
        }),
        Lang::JavaScript => None,
      };

      let ts_aliases = match language {
        Lang::TypeScript => Some(CreationPaths {
          template: TS_ALIASES_PATH.to_owned(),
          default: TS_ALIAS.to_owned(),
        }),
        Lang::JavaScript => None,
      };

      Ok(PageCreation::new(
        &config.templates,
        styles_ext.clone(),
        PageCreationImports {
          page: Some(CreationPaths {
            template: format!("{PAGE_IMPORT_PATH}{ext}"),
            default: PAGE_IMPORT.to_owned(),
          }),
          styles: CreationPaths {
            template: format!("{PAGE_BARREL}{ext}"),
            default: styles_import,
          },
          i18n: i18n_import,
          locale: i18n_locale_import,
        },
        CreationPaths {
          template: format!("{PAGE_PATH}{ext}x"),
          default: page_template,
        },
        CreationPaths {
          template: format!("{STYLES_PATH}{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: format!("{RESPONSIVE_PATH}{styles_ext}"),
          default: responsive_template,
        },
        PageCreationAliases {
          config: CreationPaths {
            template: format!("{PAGE_CONFIG_PATH}{ext}"),
            default: VITE_CONFIG.to_owned(),
          },
          config_aliases: CreationPaths {
            template: format!("{PAGE_ALIASES_PATH}{ext}"),
            default: VITE_ALIAS.to_owned(),
          },
          ts_file,
          ts_aliases,
        },
        Some(CreationPaths {
          template: format!("{PAGE_ROUTER_PATH}{ext}"),
          default: ROUTER.to_owned()
        }),
        Some(CreationPaths {
          template: format!("{PAGE_ROTUE_PATH}{ext}"),
          default: ROUTE.to_owned()
        }),
        None,
        proptypes,
        i18n_templates,
        PageCreationExports {
          config: format!(".{VITE_CONFIG_PATH}{ext}"),
          page: format!("{path}{PAGE_FILE}{ext}x"),
          barrel_styles: format!("{path_ui}{STYLES_PATH}{INDEX_PATH}{ext}"),
          styles: styles_export,
          responsive: responsive_export,
          i18n: i18n_context,
          barrel_i18n: i18n_barrel,
          proptypes: proptypes_export,
          locales: Some(i18n_locales.into_iter().map(|locale| {
            format!("{path_locales}/{locale}/{}{JSON_EXT}", name.namespace)
          }).collect()),
          router: Some(format!("{}{PAGE_ROUTER_PATH}{ext}x", &paths.routes)),
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::page::{
        PAGE_CSS,
        PAGE_STYLED,
        BARREL_STYLES_CSS,
        BARREL_STYLES_STYLED,
        I18N_IMPORT,
        SCRIPT_CSS,
        SCRIPT_TS_CSS,
        SCRIPT_STYLED,
        SCRIPT_TS_STYLED,
        STYLES_EMOTION,
        STYLES_EMOTION_RESPONSIVE,
        PROPTYPES,
        LOCALE,
        I18N,
        SVELTE_CONFIG,
        SVELTE_ALIAS,
      };

      create_dir_all(&paths.pages).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });

      let i18n_templates = if i18n {
        Some(PageCreationI18n {
          locale: CreationPaths {
            template: LOCALE_FILE.to_owned(),
            default: LOCALE.to_owned()
          },
          context: CreationPaths {
            template: format!("{I18N_PATH}{ext}"),
            default: I18N.to_owned()
          }
        })
      } else { None };

      let i18n_barrel = match i18n {
        true => Some(path_i18n.replace("i18n", format!("index{ext}").as_str())),
        false => None,
      };

      let i18n_context = match i18n {
        true => Some(format!("{path_i18n}{I18N_SVELTE_FILE}{ext}")),
        false => None,
      };

      let i18n_import = match i18n {
        true => Some(CreationPaths {
          template: format!("{I18N_CONTEXT}{ext}"),
          default: I18N_IMPORT.to_owned()
        }),
        false => None,
      };

      let page_template = match styles {
        Styles::Emotion | Styles::StyledComponents => PAGE_STYLED.to_owned(),
        _ => PAGE_CSS.to_owned(),
      };

      let script_template = match styles {
        Styles::Emotion | Styles::StyledComponents => match language {
          Lang::TypeScript => SCRIPT_TS_STYLED.to_owned(),
          Lang::JavaScript => SCRIPT_STYLED.to_owned(),
        },
        _ => match language {
          Lang::TypeScript => SCRIPT_TS_CSS.to_owned(),
          Lang::JavaScript => SCRIPT_CSS.to_owned(),
        }
      };

      let styles_import = match styles {
        Styles::Emotion | Styles::StyledComponents => BARREL_STYLES_STYLED.to_owned(),
        _ => BARREL_STYLES_CSS.to_owned(),
      };

      let styles_template = match styles {
        Styles::Emotion => STYLES_EMOTION.to_owned(),
        Styles::StyledComponents => STYLES_EMOTION.to_owned(),
        Styles::CSS => STYLES_CSS.to_owned().replace(NAME_PASCAL, NAMESPACE),
        Styles::LESS => STYLES_LESS.to_owned().replace(NAME_PASCAL, NAMESPACE),
        Styles::SCSS => STYLES_SASS.to_owned().replace(NAME_PASCAL, NAMESPACE),
        Styles::Stylus => STYLES_STYLUS.to_owned().replace(NAME_PASCAL, NAMESPACE),
        Styles::PostCSS => STYLES_POSTCSS.to_owned().replace(NAME_PASCAL, NAMESPACE),
      };

      let responsive_template = match styles {
        Styles::Emotion => STYLES_EMOTION_RESPONSIVE.to_owned(),
        Styles::StyledComponents => STYLES_EMOTION_RESPONSIVE.to_owned(),
        Styles::CSS => STYLES_CSS_RESPONSIVE.to_owned(),
        Styles::LESS => STYLES_LESS_RESPONSIVE.to_owned(),
        Styles::SCSS => STYLES_SASS_RESPONSIVE.to_owned(),
        Styles::Stylus => STYLES_STYLUS_RESPONSIVE.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS_RESPONSIVE.to_owned(),
      };

      let proptypes = match language {
        Lang::TypeScript => Some(CreationPaths {
          template: format!("{PROPTYPES_PATH}{ext}"),
          default: PROPTYPES.to_string(),
        }),
        Lang::JavaScript => None,
      };

      Ok(PageCreation::new(
        &config.templates,
        styles_ext.clone(),
        PageCreationImports {
          page: None,
          styles: CreationPaths {
            template: format!("{PAGE_BARREL}{ext}"),
            default: styles_import,
          },
          i18n: i18n_import,
          locale: i18n_locale_import,
        },
        CreationPaths {
          template: format!("{PAGE_PATH}{SVELTE_EXT}"),
          default: page_template,
        },
        CreationPaths {
          template: format!("{STYLES_PATH}{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: format!("{RESPONSIVE_PATH}{styles_ext}"),
          default: responsive_template,
        },
        PageCreationAliases {
          config: CreationPaths {
            template: format!("{PAGE_CONFIG_PATH}{ext}"),
            default: SVELTE_CONFIG.to_owned(),
          },
          config_aliases: CreationPaths {
            template: format!("{PAGE_ALIASES_PATH}{ext}"),
            default: SVELTE_ALIAS.to_owned(),
          },
          ts_file: None,
          ts_aliases: None,
        },
        None,
        None,
        Some(CreationPaths {
          template: format!("{SCRIPT_PATH}{ext}{SVELTE_EXT}"),
          default: script_template,
        }),
        proptypes,
        i18n_templates,
        PageCreationExports {
          config: format!(".{SVELTE_CONFIG_PATH}"),
          page: format!("{path}{PAGE_FILE}{SVELTE_EXT}"),
          barrel_styles: format!("{path_ui}{STYLES_PATH}{INDEX_PATH}{ext}"),
          styles: styles_export,
          responsive: responsive_export,
          i18n: i18n_context,
          barrel_i18n: i18n_barrel,
          proptypes: proptypes_export,
          locales: Some(i18n_locales.into_iter().map(|locale| {
            format!("{path_locales}/{locale}/{}{JSON_EXT}", name.namespace)
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