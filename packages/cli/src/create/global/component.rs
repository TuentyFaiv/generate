use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::technologies::enums::{Lang, Tool, Styles};
use crate::statics;
use crate::statics::OK;

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
  ComponentCreation,
  ComponentCreationExports,
};
use super::constants::{
  PROPTYPES_PATH,
  PROPTYPES_EXT,
  COMPONENT_PATH,
  SCRIPT_PATH,
  STYLES_PATH,
  RESPONSIVE_PATH,
  INDEX_PATH,
  IMPORT_PATH,
  SVELTE_EXT,
  // VUE_EXT,
  STYLES_EXT,
  RESPONSIVE_EXT,
};

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, tool, tool_type, language, styles, .. } = answers;
  let ext = language.extension();
  let styles_ext = styles.extension(language);
  let name = &name.pascal;

  let full_path = match tool_type {
    Some(tool_type) => format!("{path}/{tool_type}/{name}"),
    None => format!("{path}/{name}")
  };

  let styles_export = match styles {
    Styles::Emotion | Styles::StyledComponents => format!("{full_path}/{name}{STYLES_EXT}{styles_ext}"),
    _ => format!("{full_path}/{name}{styles_ext}"),
  };

  let responsive_export = match styles {
    Styles::Emotion | Styles::StyledComponents => format!("{full_path}/{name}{STYLES_EXT}{RESPONSIVE_EXT}{styles_ext}"),
    _ => format!("{full_path}/{name}{RESPONSIVE_EXT}{styles_ext}"),
  };

  let proptypes_export = match language {
    Lang::TypeScript => Some(format!("{full_path}/{name}{PROPTYPES_EXT}{ext}")),
    Lang::JavaScript => None,
  };

  if tool != &Tool::Vanilla {
    create_dir_all(&full_path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let component = match tool {
    Tool::React => {
      use statics::react::component::{
        IMPORT,
        COMPONENT_CSS,
        COMPONENT_TS_CSS,
        COMPONENT_STYLED,
        COMPONENT_TS_STYLED,
        PROPTYPES,
        STYLES_EMOTION,
        STYLES_EMOTION_RESPONSIVE,
        STYLES_STYLED,
        STYLES_STYLED_RESPONSIVE,
      };

      let component_template = match styles {
        Styles::Emotion | Styles::StyledComponents => match language {
          Lang::TypeScript => COMPONENT_TS_STYLED.to_owned(),
          Lang::JavaScript => COMPONENT_STYLED.to_owned(),
        },
        _ => match language {
          Lang::TypeScript => COMPONENT_TS_CSS.to_owned(),
          Lang::JavaScript => COMPONENT_CSS.to_owned(),
        },
      };

      let styles_template = match styles {
        Styles::Emotion => STYLES_EMOTION.to_owned(),
        Styles::StyledComponents => STYLES_STYLED.to_owned(),
        Styles::CSS => STYLES_CSS.to_owned(),
        Styles::LESS => STYLES_LESS.to_owned(),
        Styles::SCSS => STYLES_SASS.to_owned(),
        Styles::Stylus => STYLES_STYLUS.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS.to_owned(),
        _ => STYLES_CSS.to_owned(),
      };

      let responsive_template = match styles {
        Styles::Emotion => STYLES_EMOTION_RESPONSIVE.to_owned(),
        Styles::StyledComponents => STYLES_STYLED_RESPONSIVE.to_owned(),
        Styles::CSS => STYLES_CSS_RESPONSIVE.to_owned(),
        Styles::LESS => STYLES_LESS_RESPONSIVE.to_owned(),
        Styles::SCSS => STYLES_SASS_RESPONSIVE.to_owned(),
        Styles::Stylus => STYLES_STYLUS_RESPONSIVE.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS_RESPONSIVE.to_owned(),
        _ => STYLES_CSS_RESPONSIVE.to_owned(),
      };
      
      let proptypes = match language {
        Lang::TypeScript => Some(CreationPaths {
          template: format!("{PROPTYPES_PATH}{ext}"),
          default: PROPTYPES.to_owned(),
        }),
        Lang::JavaScript => None,
      };

      Ok(ComponentCreation::new(
        &config.templates,
        styles_ext.to_string(),
        CreationPaths {
          template: format!("{IMPORT_PATH}{ext}"),
          default: IMPORT.to_owned(),
        },
        CreationPaths {
          template: format!("{STYLES_PATH}{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: format!("{COMPONENT_PATH}{ext}x"),
          default: component_template,
        },
        CreationPaths {
          template: format!("{RESPONSIVE_PATH}{styles_ext}"),
          default: responsive_template,
        },
        proptypes,
        None,
        ComponentCreationExports {
          barrel: full_path.replace(format!("/{name}").as_str(), format!("{INDEX_PATH}{ext}").as_str()),
          component: format!("{full_path}/{name}{ext}x"),
          styles: styles_export,
          responsive: responsive_export,
          proptypes: proptypes_export,
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::component::{
        IMPORT,
        COMPONENT_CSS,
        COMPONENT_STYLED,
        PROPTYPES,
        SCRIPT_CSS,
        SCRIPT_TS_CSS,
        SCRIPT_STYLED,
        SCRIPT_TS_STYLED,
        STYLES_EMOTION,
        STYLES_EMOTION_RESPONSIVE,
      };

      let component_template = match styles {
        Styles::Emotion | Styles::StyledComponents => COMPONENT_STYLED.to_owned(),
        _ => COMPONENT_CSS.to_owned(),
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
        Styles::CSS => STYLES_CSS.to_owned(),
        Styles::LESS => STYLES_LESS.to_owned(),
        Styles::SCSS => STYLES_SASS.to_owned(),
        Styles::Stylus => STYLES_STYLUS.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS.to_owned(),
        _ => STYLES_CSS.to_owned(),
      };

      let responsive_template = match styles {
        Styles::Emotion => STYLES_EMOTION_RESPONSIVE.to_owned(),
        Styles::StyledComponents => STYLES_EMOTION_RESPONSIVE.to_owned(),
        Styles::CSS => STYLES_CSS_RESPONSIVE.to_owned(),
        Styles::LESS => STYLES_LESS_RESPONSIVE.to_owned(),
        Styles::SCSS => STYLES_SASS_RESPONSIVE.to_owned(),
        Styles::Stylus => STYLES_STYLUS_RESPONSIVE.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS_RESPONSIVE.to_owned(),
        _ => STYLES_CSS_RESPONSIVE.to_owned(),
      };

      let proptypes = match language {
        Lang::TypeScript => Some(CreationPaths {
          template: format!("{PROPTYPES_PATH}{ext}"),
          default: PROPTYPES.to_owned(),
        }),
        Lang::JavaScript => None,
      };

      Ok(ComponentCreation::new(
        &config.templates,
        styles_ext.to_string(),
        CreationPaths {
          template: format!("{IMPORT_PATH}{ext}"),
          default: IMPORT.to_owned(),
        },
        CreationPaths {
          template: format!("{STYLES_PATH}{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: format!("{COMPONENT_PATH}{SVELTE_EXT}"),
          default: component_template,
        },
        CreationPaths {
          template: format!("{RESPONSIVE_PATH}{styles_ext}"),
          default: responsive_template,
        },
        proptypes,
        Some(CreationPaths {
          template: format!("{SCRIPT_PATH}{ext}{SVELTE_EXT}"),
          default: script_template
        }),
        ComponentCreationExports {
          barrel: full_path.replace(format!("/{name}").as_str(), format!("{INDEX_PATH}{ext}").as_str()),
          component: format!("{full_path}/{name}{SVELTE_EXT}"),
          styles: styles_export,
          responsive: responsive_export,
          proptypes: proptypes_export,
        }
      ))
    },
    _ => Err(anyhow!(error.clone())),
  };

  match global.generate_component(component?) {
    Ok(_) => {
      done();
      Ok(format!("{} {}", OK, style(format!("Component {name} created at {full_path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}