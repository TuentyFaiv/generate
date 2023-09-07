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

use super::CLIGlobalCreation;
use super::structs::{
  CreationPaths,
  LayoutCreation,
  LayoutCreationExports,
};
use super::constants::{
  INDEX_PATH,
  PROPTYPES_PATH,
  STYLES_PATH,
  STYLES_EXT,
  RESPONSIVE_PATH,
  RESPONSIVE_EXT,
  LAYOUT_PATH,
  LAYOUT_PROPS_PATH,
  LAYOUT_FILE,
  LAYOUT_SVELTE_EXT,
  LAYOUT_REACT_EXT,
  LAYOUT_BARREL,
  SVELTE_EXT,
  SCRIPT_PATH,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, tool, language, styles, .. } = answers;
  let paths = &config.paths;
  let ext = language.to_extension();
  let styles_ext = styles.to_extension(language);
  let name_pascal = &name.pascal;

  let path_ui = format!("{}/{}", paths.ui, name.namespace);
  let mut path_proptypes = String::new();

  let styles_name = match tool {
    Tool::React => name_pascal,
    Tool::Svelte => &name.namespace,
    Tool::Vanilla => name_pascal,
  };

  let styles_tool_ext = match tool {
    Tool::React => LAYOUT_REACT_EXT,
    Tool::Svelte => LAYOUT_SVELTE_EXT,
    Tool::Vanilla => LAYOUT_REACT_EXT,
  };

  let styles_export = match styles {
    Styles::Emotion | Styles::StyledComponents => format!("{path_ui}{STYLES_PATH}/{styles_name}{styles_tool_ext}{STYLES_EXT}{styles_ext}"),
    _ => format!("{path_ui}{STYLES_PATH}/{styles_name}{styles_tool_ext}{styles_ext}"),
  };

  let responsive_export = match styles {
    Styles::Emotion | Styles::StyledComponents => format!("{path_ui}{STYLES_PATH}/{styles_name}{styles_tool_ext}{STYLES_EXT}{RESPONSIVE_EXT}{styles_ext}"),
    _ => format!("{path_ui}{STYLES_PATH}/{styles_name}{styles_tool_ext}{RESPONSIVE_EXT}{styles_ext}"),
  };

  create_dir_all(path).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });
  create_dir_all(format!("{path_ui}{STYLES_PATH}")).unwrap_or_else(|why| {
    println!("! {:?}", why.kind());
  });

  match language {
    Lang::TypeScript => {
      path_proptypes = format!("{}{LAYOUT_PROPS_PATH}", paths.types);
      create_dir_all(&path_proptypes).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
      });
    },
    Lang::JavaScript => {},
  };

  let layout = match tool {
    Tool::React => {
      use statics::react::layout::{
        LAYOUT_CSS,
        LAYOUT_TS_CSS,
        LAYOUT_STYLED,
        LAYOUT_TS_STYLED,
        STYLES_EMOTION,
        STYLES_EMOTION_RESPONSIVE,
        STYLES_STYLED,
        STYLES_STYLED_RESPONSIVE,
        PROPTYPES,
        BARREL_STYLES_CSS,
        BARREL_STYLES_STYLED
      };

      let layout_template = match styles {
        Styles::Emotion | Styles::StyledComponents => match language {
          Lang::TypeScript => LAYOUT_TS_STYLED.to_owned(),
          Lang::JavaScript => LAYOUT_STYLED.to_owned(),
        },
        _ => match language {
          Lang::TypeScript => LAYOUT_TS_CSS.to_owned(),
          Lang::JavaScript => LAYOUT_CSS.to_owned(),
        },
      };

      let styles_template = match styles {
        Styles::Emotion => STYLES_EMOTION.to_owned(),
        Styles::StyledComponents => STYLES_STYLED.to_owned(),
        Styles::CSS => STYLES_CSS.to_owned().replace(NAME_PASCAL, format!("{NAME_PASCAL}{styles_tool_ext}").as_str()),
        Styles::LESS => STYLES_LESS.to_owned().replace(NAME_PASCAL, format!("{NAME_PASCAL}{styles_tool_ext}").as_str()),
        Styles::SCSS => STYLES_SASS.to_owned().replace(NAME_PASCAL, format!("{NAME_PASCAL}{styles_tool_ext}").as_str()),
        Styles::Stylus => STYLES_STYLUS.to_owned().replace(NAME_PASCAL, format!("{NAME_PASCAL}{styles_tool_ext}").as_str()),
        Styles::PostCSS => STYLES_POSTCSS.to_owned().replace(NAME_PASCAL, format!("{NAME_PASCAL}{styles_tool_ext}").as_str()),
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
        Lang::TypeScript =>  Some(CreationPaths {
          template: format!("{PROPTYPES_PATH}{ext}"),
          default: PROPTYPES.to_string(),
        }),
        Lang::JavaScript => None,
      };

      let proptypes_export = match language {
        Lang::TypeScript => Some(format!("{path_proptypes}/{}{ext}", name.namespace)),
        Lang::JavaScript => None,
      };

      let barrel_styles = match styles {
        Styles::Emotion | Styles::StyledComponents => BARREL_STYLES_STYLED.to_owned(),
        _ => BARREL_STYLES_CSS.to_owned(),
      };

      Ok(LayoutCreation::new(
        &config.templates,
        styles_ext.clone(),
        CreationPaths {
          template: format!("{LAYOUT_BARREL}{styles_ext}"),
          default: barrel_styles,
        },
        CreationPaths {
          template: format!("{LAYOUT_PATH}{ext}x"),
          default: layout_template,
        },
        CreationPaths {
          template: format!("{STYLES_PATH}{styles_ext}"),
          default: styles_template
        },
        CreationPaths {
          template: format!("{RESPONSIVE_PATH}{styles_ext}"),
          default: responsive_template,
        },
        proptypes,
        None,
        LayoutCreationExports {
          barrel_styles: format!("{path_ui}{STYLES_PATH}{INDEX_PATH}{ext}"),
          layout: format!("{path}{LAYOUT_FILE}{ext}x"),
          styles: styles_export,
          responsive: responsive_export,
          proptypes: proptypes_export,
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::layout::{
        LAYOUT_CSS,
        LAYOUT_STYLED,
        SCRIPT_CSS,
        SCRIPT_TS_CSS,
        SCRIPT_STYLED,
        SCRIPT_TS_STYLED,
        PROPTYPES,
        STYLES_EMOTION,
        STYLES_EMOTION_RESPONSIVE,
        BARREL_STYLES_CSS,
        BARREL_STYLES_STYLED
      };

      let layout_template = match styles {
        Styles::Emotion | Styles::StyledComponents => LAYOUT_STYLED.to_owned(),
        _ => LAYOUT_CSS.to_owned(),
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

      let styles_template = match styles {
        Styles::Emotion => STYLES_EMOTION.to_owned(),
        Styles::StyledComponents => STYLES_EMOTION.to_owned(),
        Styles::CSS => STYLES_CSS.to_owned().replace(NAME_PASCAL, format!("{NAMESPACE}{styles_tool_ext}").as_str()),
        Styles::LESS => STYLES_LESS.to_owned().replace(NAME_PASCAL, format!("{NAMESPACE}{styles_tool_ext}").as_str()),
        Styles::SCSS => STYLES_SASS.to_owned().replace(NAME_PASCAL, format!("{NAMESPACE}{styles_tool_ext}").as_str()),
        Styles::Stylus => STYLES_STYLUS.to_owned().replace(NAME_PASCAL, format!("{NAMESPACE}{styles_tool_ext}").as_str()),
        Styles::PostCSS => STYLES_POSTCSS.to_owned().replace(NAME_PASCAL, format!("{NAMESPACE}{styles_tool_ext}").as_str()),
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
          default: PROPTYPES.to_owned(),
        }),
        Lang::JavaScript => None,
      };

      let proptypes_export = match language {
        Lang::TypeScript => Some(format!("{path_proptypes}/{}{ext}", name.namespace)),
        Lang::JavaScript => None,
      };

      let barrel_styles = match styles {
        Styles::Emotion | Styles::StyledComponents => BARREL_STYLES_STYLED.to_owned(),
        _ => BARREL_STYLES_CSS.to_owned(),
      };

      Ok(LayoutCreation::new(
        &config.templates,
        styles_ext.clone(),
        CreationPaths {
          template: format!("{LAYOUT_BARREL}{styles_ext}"),
          default: barrel_styles,
        },
        CreationPaths {
          template: format!("{LAYOUT_PATH}{SVELTE_EXT}"),
          default: layout_template,
        },
        CreationPaths {
          template: format!("{STYLES_PATH}{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: format!("{RESPONSIVE_PATH}{styles_ext}"),
          default: responsive_template,
        },
        proptypes,
        Some(CreationPaths {
          template: format!("{SCRIPT_PATH}{ext}{SVELTE_EXT}"),
          default: script_template,
        }),
        LayoutCreationExports {
          barrel_styles: format!("{path_ui}{STYLES_PATH}{INDEX_PATH}{ext}"),
          layout: format!("{path}{LAYOUT_FILE}{SVELTE_EXT}"),
          styles: styles_export,
          responsive: responsive_export,
          proptypes: proptypes_export,
        }
      ))
    }
    Tool::Vanilla => Err(anyhow!(error.clone())),
  };

  match global.generate_layout(layout?) {
    Ok(_) => {
      done();
      Ok(format!( "{} {}", OK, style(format!("Layout {name_pascal} created at {path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}