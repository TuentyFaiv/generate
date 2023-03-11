use anyhow::{Result};
use clap::Parser;
// use indicatif::ProgressBar;

mod utils;
mod cli;
mod statics;
mod templates;
mod create;

use crate::cli::{questions, Args, msg};
use crate::statics::{NOT_IMPLEMENTED};
use crate::templates::get_templates;
use crate::create::{global, react};

pub fn main() -> Result<()> {
	// env_logger::init();

	let args = Args::parse();

	let templates = get_templates();

	let answers = questions::make(&args)?;

	if !answers.accept {
		return Ok(());
	}

	let template = match templates.get(&answers.template.as_str()) {
		None => {
			[].to_vec()
		},
		Some(&option) => {
			let options: Vec<&str> = option.split('-').collect();

			options
		}
	};

	let create_project = template.contains(&"repo");
	// let create_library = template.contains(&"library");
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
	
	if create_hoc {
		react::hoc::make(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.path
		)?;
	}

	if create_hook {
		react::hook::make(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.path
		)?;
	}
	if create_context {
		react::context::make(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.path
		)?;
	}

	if create_schema {
		global::schema::make(
			&answers.name,
			&answers.tool_type,
			&answers.path
		)?;
	}

	if create_service {
		global::service::make(
			&answers.name,
			&answers.tool_type,
			&answers.path
		)?;
	}

	if create_page {
		global::page::make(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.path
		)?;
	}

	if create_layout {
		global::layout::make(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.path
		)?;
	}

	if create_component {
		global::component::make(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.arch_type,
			&answers.path
		)?;
	}

	if create_project {
		global::project::make(
			&template,
			&answers.name,
			&answers.path,
			&answers.tool,
			&answers.arch
		);
	}

	println!("");

	// let pb = ProgressBar::new(100);

	// for i in 0..100 {
	// 	pb.println(format!("[+] finished #{}", i));
	// 	pb.inc(1);
	// }

	// pb.finish_with_message("Done");

	Ok(())
}
