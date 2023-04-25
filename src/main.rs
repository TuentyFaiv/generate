use anyhow::{Result};
use clap::Parser;

mod utils;
mod cli;
mod statics;
mod templates;
mod create;
mod config;

use crate::cli::{questions, Args, msg};
use crate::statics::{NOT_IMPLEMENTED};
use crate::templates::get_templates;
use crate::create::{global, react};

pub fn main() -> Result<()> {
	let args = Args::parse();

	let answers = questions::make(&args)?;	
	
	if !answers.accept {
		return Ok(());
	}

	let templates = get_templates();

	let template = match templates.get(&answers.template.as_str()) {
		None => [].to_vec(),
		Some(&option) => option.split('-').collect()
	};

	let create_project = template.contains(&"repo");
	let create_library = template.contains(&"library");
	let create_component = template.contains(&"component");
	let create_hoc = template.contains(&"hoc");
	let create_hook = template.contains(&"hook");
	let create_context = template.contains(&"context");
	let create_page = template.contains(&"page");
	let create_layout = template.contains(&"layout");
	let create_schema = template.contains(&"schema");
	let create_service = template.contains(&"service");
	let notexist = template.contains(&"notimplemented");

	if notexist {
		msg(&format!("{} Option not implemented yet", NOT_IMPLEMENTED));
		return Ok(());
	}
	
	// React generation
	if create_hoc {
		react::hoc::make(&answers)?;
	}
	if create_hook {
		react::hook::make(&answers)?;
	}
	if create_context {
		react::context::make(&answers)?;
	}

	// Global generation
	if create_schema {
		global::schema::make(&answers)?;
	}
	if create_service {
		global::service::make(&answers)?;
	}
	if create_page {
		global::page::make(&answers)?;
	}
	if create_layout {
		global::layout::make(&answers)?;
	}
	if create_component {
		global::component::make(&answers)?;
	}
	if create_project || create_library {
		global::project::make(&answers, &template, create_library)?;
	}

	println!("");

	Ok(())
}
