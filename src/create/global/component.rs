use std::fs::create_dir_all;
use anyhow::{Result, anyhow};
use console::style;

use crate::cli::{utils::done, structs::Answers};
use crate::cli::enums::{Lang, Tool, Styles};
use crate::statics;
use crate::statics::OK;

use super::CLIGlobalCreation;
use super::structs::{
  CreationPaths,
  ComponentCreation,
  ComponentCreationExports,
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

pub fn create(CLIGlobalCreation {
  answers,
  config,
  error,
  global,
  ..
}: &CLIGlobalCreation) -> Result<String> {
  let Answers { name, path, tool, tool_type, language, styles, .. } = answers;
  let is_ts = *language == Lang::TypeScript;
  let ext = language.to_extension();
  let styles_ext = styles.to_extension(language);
  let name = &name.pascal;
  let is_styled = *styles == Styles::StyledComponents || *styles == Styles::Emotion;

  let full_path = match tool_type {
    Some(tool_type) => format!("{path}/{tool_type}/{name}"),
    None => format!("{path}/{name}")
  };

  let styles_export = if is_styled {
    format!("{full_path}/{name}.styles{styles_ext}")
  } else {
    format!("{full_path}/{name}{styles_ext}")
  };

  let responsive_export = if is_styled {
    format!("{full_path}/{name}.styles.responsive{styles_ext}")
  } else {
    format!("{full_path}/{name}.responsive{styles_ext}")
  };

  if tool != &Tool::Vanilla {
    create_dir_all(&full_path).unwrap_or_else(|why| {
      println!("! {:?}", why.kind());
    });
  }

  let component = match tool {
    Tool::React => {
      use statics::react::component::{
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

      let component_template  = if is_styled {
        if is_ts { COMPONENT_TS_STYLED.to_owned() } else { COMPONENT_STYLED.to_owned() }
      } else {
        if is_ts { COMPONENT_TS_CSS.to_owned() } else { COMPONENT_CSS.to_owned() }
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
      
      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_string(),
        })
      } else { None };

      Ok(ComponentCreation::new(
        &config.templates,
        styles_ext.clone(),
        format!("export {{ default as {name} }} from \"./{name}/{name}\";\n"),
        CreationPaths {
          template: format!("/styles{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: format!("/component{ext}x"),
          default: component_template,
        },
        CreationPaths {
          template: format!("/styles.responsive{styles_ext}"),
          default: responsive_template,
        },
        proptypes,
        None,
        ComponentCreationExports {
          barrel: full_path.replace(format!("/{name}").as_str(), format!("/index{ext}").as_str()),
          component: format!("{full_path}/{name}{ext}x"),
          styles: styles_export,
          responsive: responsive_export,
          proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
        }
      ))
    },
    Tool::Svelte => {
      use statics::svelte::component::{
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

      let component_template = if is_styled {
        COMPONENT_STYLED.to_owned()
      } else {
        COMPONENT_CSS.to_owned()
      };

      let script_template = if is_styled {
        if is_ts { SCRIPT_TS_STYLED.to_owned() } else { SCRIPT_STYLED.to_owned() }
      } else {
        if is_ts { SCRIPT_TS_CSS.to_owned() } else { SCRIPT_CSS.to_owned() }
      };

      let styles_template = match styles {
        Styles::Emotion => STYLES_EMOTION.to_owned(),
        Styles::StyledComponents => STYLES_EMOTION.to_owned(),
        Styles::CSS => STYLES_CSS.to_owned(),
        Styles::LESS => STYLES_LESS.to_owned(),
        Styles::SCSS => STYLES_SASS.to_owned(),
        Styles::Stylus => STYLES_STYLUS.to_owned(),
        Styles::PostCSS => STYLES_POSTCSS.to_owned(),
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

      let proptypes = if is_ts {
        Some(CreationPaths {
          template: format!("/proptypes{ext}"),
          default: PROPTYPES.to_string(),
        })
      } else { None };
      let script = Some(CreationPaths {
        template: format!("/script{ext}.svelte"),
        default: script_template
      });

      Ok(ComponentCreation::new(
        &config.templates,
        styles_ext.clone(),
        format!("export {{ default as {name} }} from \"./{name}/{name}.svelte\";\n"),
        CreationPaths {
          template: format!("/styles{styles_ext}"),
          default: styles_template,
        },
        CreationPaths {
          template: "/component.svelte".to_string(),
          default: component_template,
        },
        CreationPaths {
          template: format!("/styles.responsive{styles_ext}"),
          default: responsive_template,
        },
        proptypes,
        script,
        ComponentCreationExports {
          barrel: full_path.replace(format!("/{name}").as_str(), format!("/index{ext}").as_str()),
          component: format!("{full_path}/{name}.svelte"),
          styles: styles_export,
          responsive: responsive_export,
          proptypes: if is_ts { Some(format!("{full_path}/{name}.proptypes{ext}")) } else { None },
        }
      ))
    },
    Tool::Vanilla => Err(anyhow!(error.clone())),
  };

  match global.generate_component(component?) {
    Ok(_) => {
      done();
      Ok(format!("{} {}", OK, style(format!("Component {name} created at {full_path}")).cyan()))
    },
    Err(error) => Err(error)
  }
}