pub mod global;
pub mod react;
pub mod svelte;
pub mod structs;
pub mod constants;

use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::statics::NOT_IMPLEMENTED;
use crate::utils::to_vec;
use crate::config::CLIConfig;
use crate::cli::enums::{ArchType, Tool};
use crate::cli::structs::Answers;

use self::global::CLIGlobalCreation;
use self::react::CLIReactCreation;
use self::svelte::CLISvelteCreation;

pub struct CLICreation {
	answers: Answers,
	config: CLIConfig,
	creators: CLICreators,
	pub error: String,
}

struct CLICreators {
	global: CLIGlobalCreation,
	svelte: CLISvelteCreation,
	react: CLIReactCreation,
}

impl CLICreation {
	pub fn new(config: CLIConfig, answers: Answers) -> Self {
		let error = format!("{} Option not implemented yet", NOT_IMPLEMENTED);
		let global = CLIGlobalCreation::new(
			config.clone(),
			answers.clone(),
			error.clone()
		);
		let react = CLIReactCreation::new(
			config.clone(),
			answers.clone(),
			error.clone()
		);
		let svelte = CLISvelteCreation::new(
			config.clone(),
			answers.clone(),
			error.clone()
		);

		Self {
			config,
			answers,
			error,
			creators: CLICreators {
				global,
				svelte,
				react,
			},
		}
	}
	pub fn create(&self) -> Result<String> {
		let template_type = self.get_template()?;
		let CLICreators { global, .. } = &self.creators;

		match template_type {
			ArchType::Project | ArchType::Library => global.make_project(),
			ArchType::Component => global.make_component(),
			ArchType::Page => global.make_page(),
			ArchType::Layout => global.make_layout(),
			ArchType::Schema => global.make_schema(),
			ArchType::Service => global.make_service(),
			// ArchType::Context => global.make_context(),
			// ArchType::Hoc => react.make_hoc(),
			// ArchType::Hook => react.make_hook(),
			// ArchType::Action => svelte.make_action(),
			// ArchType::Store => svelte.make_store(),
			_ => Err(anyhow!(self.error.clone()))
		}
	}
	fn get_template(&self) -> Result<&ArchType> {
		let tools = &self.config.tools;
		let react_options = self.config.projects_options(&Tool::React);
		let svelte_options = self.config.projects_options(&Tool::Svelte);
		let vanilla_options = self.config.projects_options(&Tool::Vanilla);
		let project_options = [react_options, svelte_options, vanilla_options].concat();
		let react_lib_options = self.config.library_options(&Tool::React);
		let svelte_lib_options = self.config.library_options(&Tool::Svelte);
		let vanilla_lib_options = self.config.library_options(&Tool::Vanilla);
		let library_options = [react_lib_options, svelte_lib_options, vanilla_lib_options].concat();
		let mut templates = HashMap::<ArchType, &Vec<String>>::new();

		let only_react: Vec<String> = to_vec(&["react"]);
		let only_svelte: Vec<String> = to_vec(&["svelte"]);
		let only_vanilla: Vec<String> = to_vec(&["vanilla"]);
		let only_svelte_and_react: Vec<String> = [only_react.clone(), only_svelte.clone()].concat();

		templates.insert(ArchType::Project, &project_options);
		templates.insert(ArchType::Library, &library_options);
		templates.insert(ArchType::Component, &tools.globals);
		templates.insert(ArchType::Hoc, &only_react);
		templates.insert(ArchType::Hook, &only_react);
		templates.insert(ArchType::Context, &only_svelte_and_react);
		templates.insert(ArchType::Action, &only_svelte);
		templates.insert(ArchType::Store, &only_svelte);
		templates.insert(ArchType::Class, &only_vanilla);
		templates.insert(ArchType::Page, &tools.globals);
		templates.insert(ArchType::Layout, &tools.globals);
		templates.insert(ArchType::Service, &tools.globals);
		templates.insert(ArchType::Schema, &tools.globals);

		let arch = &self.answers.arch;
		let tool = &self.answers.tool;
		let tool_type = &self.answers.tool_type;
		let arch_exist = templates.get(&arch);

		match arch_exist {
			None => Err(anyhow!(self.error.clone())),
			Some(template) => {
				if let Some(tool_type) = tool_type {
					if template.contains(tool_type) {
						return Ok(arch);
					}
				}
				if template.contains(&tool.to_string()) {
					return Ok(arch);
				}

				Err(anyhow!(self.error.clone()))
			}
		}
	}
}
