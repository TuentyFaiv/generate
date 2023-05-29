pub mod svelte;
pub mod react;
pub mod vanilla;
pub mod global;

use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::statics::NOT_IMPLEMENTED;
use crate::utils::to_vec;
use crate::config::CLIConfig;
use crate::cli::enums::Tool;
use crate::cli::{enums::ArchType, structs::Answers};
use crate::create::global::CLIGlobalCreation;
use crate::create::react::CLIReactCreation;
use crate::create::svelte::CLISvelteCreation;

pub struct CLITemplates {
	answers: Answers,
	config: CLIConfig,
	global_creation: CLIGlobalCreation,
	react_creation: CLIReactCreation,
	svelte_creation: CLISvelteCreation,
	pub error: String,
}

impl CLITemplates {
	pub fn new(config: CLIConfig, answers: Answers) -> Self {
		let error = format!("{} Option not implemented yet", NOT_IMPLEMENTED);
		let global_creation = CLIGlobalCreation::new(
			config.clone(),
			answers.clone(),
			error.clone()
		);
		let react_creation = CLIReactCreation::new(
			config.clone(),
			answers.clone(),
			error.clone()
		);
		let svelte_creation = CLISvelteCreation::new(
			config.clone(),
			answers.clone(),
			error.clone()
		);

		Self {
			config,
			answers,
			error,
			global_creation,
			react_creation,
			svelte_creation,
		}
	}
	pub fn create(&self) -> Result<String> {
		let template_type = self.get_template()?;
		let create_global = &self.global_creation;
		let create_react = &self.react_creation;
		let create_svelte = &self.svelte_creation;

		match template_type {
			ArchType::Project | ArchType::Library => create_global.make_project(),
			ArchType::Component => create_global.make_component(),
			ArchType::Page => create_global.make_page(),
			ArchType::Layout => create_global.make_layout(),
			ArchType::Schema => create_global.make_schema(),
			ArchType::Service => create_global.make_service(),
			ArchType::Context => create_global.make_context(),
			ArchType::Hoc => create_react.make_hoc(),
			ArchType::Hook => create_react.make_hook(),
			ArchType::Action => create_svelte.make_action(),
			ArchType::Store => create_svelte.make_store(),
			_ => Err(anyhow!(self.error.clone()))
		}
	}
	fn get_template(&self) -> Result<&ArchType> {
		let tools = self.config.get_tools();
		let react_options = self.config.projects_options(&Tool::React);
		let svelte_options = self.config.projects_options(&Tool::Svelte);
		let vanilla_options = self.config.projects_options(&Tool::Vanilla);
		let project_options = [react_options, svelte_options, vanilla_options].concat();
		let react_lib_options = self.config.library_options(&Tool::React);
		let svelte_lib_options = self.config.library_options(&Tool::Svelte);
		let vanilla_lib_options = self.config.library_options(&Tool::Vanilla);
		let library_options = [react_lib_options, svelte_lib_options, vanilla_lib_options, tools.webcomponents.clone()].concat();
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
					if template.contains(&tool_type) {
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
