pub mod svelte;
pub mod react;
pub mod vanilla;
pub mod global;

use std::collections::HashMap;
use anyhow::{Result, anyhow};

use crate::statics::NOT_IMPLEMENTED;
use crate::config::Config;
use crate::cli::msg;
use crate::cli::{enums::ArchType, questions::Answers};
use crate::create::global::CLIGlobalCreation;
use crate::create::react::CLIReactCreation;

pub struct CLITemplates<'a, C: Config> {
	answers: Answers,
	config: &'a C,
}

impl <'a, C: Config> CLITemplates<'a, C> {
	pub fn new(config: &'a C, answers: Answers) -> Self {
		Self { config, answers }
	}
	pub fn create(&self) -> Result<()> {
		let created = match self.get_template() {
			None => Err(anyhow!("Option not implemented yet")),
			Some(template_type) => {
				let create_global = CLIGlobalCreation::new(
					self.config,
					self.answers.clone()
				);
    		let create_react = CLIReactCreation::new(
					self.config,
					self.answers.clone()
				);

				match template_type {
					ArchType::Atomic | ArchType::Library => create_global.make_project(),
					ArchType::Component => create_global.make_component(),
					ArchType::Page => create_global.make_page(),
					ArchType::Layout => create_global.make_layout(),
					ArchType::Schema => create_global.make_schema(),
					ArchType::Service => create_global.make_service(),
					ArchType::Hoc => create_react.make_hoc(),
					ArchType::Hook => create_react.make_hook(),
					ArchType::Context => create_react.make_context(), // Change to global
					// ArchType::Action => create_svelte.make_action(),
					// ArchType::Store => create_svelte.make_store(),
					_ => Err(anyhow!("Option not implemented yet"))
				}
			}
		};

		Ok(match created {
			Err(error) => msg(&format!("{} {}", NOT_IMPLEMENTED, error)),
			_ => ()
		})
	}
	fn get_template(&self) -> Option<&ArchType> {
		let mut templates = HashMap::<ArchType, &[&str]>::new();

		let only_react: &[&str] = &["react", "typescript", "javascript"];
		let only_svelte: &[&str] = &[
			// "svelte",
			"typescript",
			"javascript"
		];
		let only_svelte_and_react: &[&str] = &[
			"react",
			// "svelte",
			"typescript",
			"javascript"
		];
		let only_vanilla: &[&str] = &[
			// "vanilla",
			"typescript",
			"javascript"
		];
		let all_without_vanilla: &[&str] = &[
			"react",
			"svelte",
			// "vanilla",
			"typescript",
			"javascript",
		];
		let all_globals: &[&str] = &[
			"react",
			"svelte",
			"vanilla",
			"typescript",
			"javascript",
		];

		templates.insert(ArchType::Atomic, &[
			"react",
			"react-typescript",
			// "react-javascript",
			"next-ts",
			// "next",
			// "remix-ts",
			// "remix",
			// "native-ts",
			// "native",
			"svelte",
			"sveltekit-ts",
			// "sveltekit",
			// "svelte-typescript",
			// "svelte-javascript",
			// "vanilla-typescript",
			// "vanilla-javascript",
		]);
		templates.insert(ArchType::Library, &[
			"react-typescript",
			// "react-javascript",
			"svelte",
			// "sveltekit-ts",
			// "sveltekit",
			// "webcomponent-ts",
			// "webcomponent",
			// "svelte-typescript",
			// "svelte-javascript",
			"vanilla-typescript",
			// "vanilla-javascript",
		]);
		templates.insert(ArchType::Component, all_without_vanilla);
		templates.insert(ArchType::Hoc, only_react);
		templates.insert(ArchType::Hook, only_react);
		templates.insert(ArchType::Context, only_svelte_and_react);
		templates.insert(ArchType::Action, only_svelte);
		templates.insert(ArchType::Store, only_svelte);
		templates.insert(ArchType::Class, only_vanilla);
		templates.insert(ArchType::Page, all_without_vanilla);
		templates.insert(ArchType::Layout, all_without_vanilla);
		templates.insert(ArchType::Service, all_globals);
		templates.insert(ArchType::Schema, all_globals);

		let arch = &self.answers.arch;
		let tool = &self.answers.tool;
		let tool_type = &self.answers.tool_type;
		let arch_exist = templates.get(&arch);

		match arch_exist {
			None => None,
			Some(template) => {
				let to_search = format!("{}-{}", tool, tool_type);

				if template.contains(&to_search.as_str())
					|| (template.contains(&tool.as_str()) && template.contains(&tool_type.as_str())) {
					return Some(arch);
				}
				return None;
			}
		}
	}
}
