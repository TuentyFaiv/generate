pub mod utils;
pub mod actions;
pub mod enums;
pub mod structs;

use anyhow::{Result, anyhow};

use crate::config::CLIConfig;
use crate::utils::to_vec;

use self::utils::{show_namespaces, input, choose_option, arg_or, sure};
use self::enums::{ArchType, Tool, Lang, Styles};
use self::structs::{
  Args,
  Answers,
  AnswersName,
  AnswersToolType,
  QuestionToolType,
};

pub trait Questions {
  fn ask(&self) -> Result<Answers>;
}

pub struct CLIQuestions {
  args: Args,
  config: CLIConfig,
}

impl CLIQuestions {
  pub fn new(config: CLIConfig, args: Args) -> Self {
    Self { args, config }
  }
  fn ask_arch(&self, tool: &Tool) -> Result<ArchType> {
    let archs = self.config.archs.clone();
    let mut options_archs = match tool {
      Tool::React => [archs.globals, archs.react].concat(),
      Tool::Svelte => [archs.globals, archs.svelte].concat(),
      Tool::Vanilla => [archs.globals, archs.vanilla].concat(),
    };
    options_archs.sort();

    let arch = ArchType::parse(&arg_or(
      "Choose a piece:", 
      self.args.arch.clone(),
      &options_archs
    )?);

    Ok(arch)
  }
  fn ask_tool_type(&self, arch: &ArchType, tool: &Tool) -> Result<AnswersToolType> {
    let tools = self.config.tools.clone();
    let lang_question = QuestionToolType { prompt: "Choose language:", tools: self.config.langs.clone() };

    let tool_question = match arch {
      ArchType::Project => match tool {
        Tool::React => Some(QuestionToolType {
          prompt: "Choose react project:",
          tools: self.config.projects_options(&Tool::React)
        }),
        Tool::Svelte => Some(QuestionToolType {
          prompt: "Choose svelte project:",
          tools: self.config.projects_options(&Tool::Svelte)
        }),
        Tool::Vanilla => Some(QuestionToolType {
          prompt: "Choose vanilla project:",
          tools: self.config.projects_options(&Tool::Vanilla)
        }),
      },
      ArchType::Library => match tool {
        Tool::React => {
          Some(QuestionToolType {
            prompt: "Choose react library:",
            tools: self.config.library_options(&Tool::React)
          })
        },
        Tool::Svelte => {
          Some(QuestionToolType {
            prompt: "Choose svelte library:",
            tools: [self.config.library_options(&Tool::Svelte), tools.webcomponents].concat()
          })
        },
        Tool::Vanilla => {
          Some(QuestionToolType {
            prompt: "Choose vanilla library:",
            tools: [self.config.library_options(&Tool::Svelte), tools.webcomponents].concat()
          })
        }
      },
      ArchType::Component => {
        Some(QuestionToolType { prompt: "Choose component type:", tools: tools.components })
      }
      _ => None
    };

    let tool_type = match tool_question {
      Some(tool_question) => {
        if tool_question.tools.len() == 0 {
          Err(anyhow!("No options available for this tool"))
        } else {
          let option = choose_option(tool_question.prompt, &tool_question.tools)?;
          if option == "custom" {
            Ok(Some(input("Custom type:", "component")?))
          } else {
            Ok(Some(option))
          }
        }
      },
      None => Ok(None)
    };

    Ok(AnswersToolType {
      tool_type: tool_type?,
      language: Lang::parse(&arg_or(
        lang_question.prompt,
        self.args.language.clone(),
        &lang_question.tools)?
      ),
    })
  }
  fn ask_name(&self, arch: &ArchType) -> Result<AnswersName> {
    let name = match &self.args.name {
      None => match arch {
        ArchType::Component => input("Component name:", "component")?,
        ArchType::Hoc => input("Hoc name:", "hoc")?,
        ArchType::Hook => input("Hook name:", "hook")?,
        ArchType::Context => input("Context name:", "context")?,
        ArchType::Service => input("Service name:", "service")?,
        ArchType::Schema => input("Schema name:", "schema")?,
        ArchType::Action => input("Action name:", "action")?,
        ArchType::Store => input("Store name:", "store")?,
        ArchType::Class => input("Class name:", "class")?,
        ArchType::Project => input("Project name:", "demo")?,
        ArchType::Library => input("Library name:", "library")?,
        ArchType::Page | ArchType::Layout => "sharing".to_owned(),
      }
      Some(exist) => exist.clone()
    };

    Ok(AnswersName::new(&name))
  }
  fn ask_path(&self, arch: &ArchType, tool: &Tool, answer_name: &mut AnswersName) -> Result<String> {
    let paths = self.config.paths.clone();

    let path = match &self.args.path {
      None => match arch {
        ArchType::Hoc => paths.hocs,
        ArchType::Hook =>  match choose_option("Which type is:", &to_vec(&["global", "internal"]))?.as_str() {
          "internal" => {
            let short_path = show_namespaces(&[&paths.ui].to_vec())?;
            format!("{}/{}/{}", paths.ui, short_path, paths.hooks.internal)
          },
          _ => paths.hooks.global
        },
        ArchType::Page | ArchType::Layout => {
          let short_path = show_namespaces(&[&paths.ui].to_vec())?;
          answer_name.change(&short_path);

          let full_path = match tool {
            Tool::Svelte => {
              answer_name.set_namespace(&answer_name.lower.clone());
              if &answer_name.lower == "home" || &answer_name.lower == "index" {
                paths.pages
              } else {
                format!("{}/{}", paths.pages, answer_name.lower)
              }
            },
            _ => {
              answer_name.set_namespace(&answer_name.lower.clone());
              format!("{}/{}", paths.ui, answer_name.lower)
            }
          };

          full_path
        }
        ArchType::Component => {
          let short_path = show_namespaces(&[&paths.ui].to_vec())?;

          format!("{}/{}", paths.ui, short_path)
        },
        ArchType::Project | ArchType::Library => input("Proyect path:", &paths.get_root(&answer_name.original))?,
        ArchType::Context => format!("{}/{}", paths.contexts, answer_name.lower),
        ArchType::Service | ArchType::Schema => {
          let to = if *arch == ArchType::Service { paths.services } else { paths.schemas };
          let short_path = show_namespaces(&[&paths.ui, &to].to_vec())?;
          format!("{}/{}", to, short_path)
        },
        ArchType::Action => input("Choose location:", &paths.actions)?,
        ArchType::Store => input("Choose location:", &paths.stores)?,
        ArchType::Class => input("Choose location:", &format!("{}/{}", paths.classes, answer_name.lower))?,
      }
      Some(exist) => {
        if *arch == ArchType::Page || *arch == ArchType::Layout {
          answer_name.set_namespace(exist);
        }
        exist.clone()
      }
    };

    if *arch != ArchType::Page && *arch != ArchType::Layout {
      answer_name.set_namespace(&path);
    }
    Ok(path)
  }
  fn ask_styles(&self, arch: &ArchType) -> Result<Styles> {
    match arch {
      ArchType::Component
      | ArchType::Layout
      | ArchType::Page => {
        let styles =  Styles::parse(&arg_or(
          "Choose style:",
          self.args.styles.clone(),
          &self.config.styles)?
        );
        Ok(styles)
      },
      _ => Ok(Styles::CSS)
    }
  }
  fn ask_sure(&self) -> Result<bool> {
    let accept = match self.args.sure {
      Some(accept) => accept,
      None => sure()?
    };
    Ok(accept)
  }
}

impl Questions for CLIQuestions {
  fn ask(&self) -> Result<Answers> {
    let tools = &self.config.tools;
    let tool = Tool::parse(&arg_or(
      "Choose a tool:",
      self.args.tool.clone(),
      &tools.globals
    )?);

    let arch = self.ask_arch(&tool)?;

    let AnswersToolType { tool_type, language } = self.ask_tool_type(&arch, &tool)?;

    let styles = self.ask_styles(&arch)?;

    let mut name = self.ask_name(&arch)?;

    let path = self.ask_path(&arch, &tool, &mut name)?;

    let accept = self.ask_sure()?;

    Ok(Answers {
      name,
      path,
      tool,
      tool_type,
      language,
      styles,
      arch,
      accept,
    })
  }
}
