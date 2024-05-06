pub mod global;
pub mod react;
pub mod svelte;
pub mod structs;
pub mod constants;

use anyhow::{Result, anyhow};

use crate::statics::NOT_IMPLEMENTED;
use crate::config::CLIConfig;
use crate::technologies::enums::ArchType;
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
		let CLICreators { global, .. } = &self.creators;

		match self.answers.arch {
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
}
