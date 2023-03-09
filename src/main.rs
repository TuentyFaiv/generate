use anyhow::{Result};
use clap::Parser;
// use indicatif::ProgressBar;

mod cli;
mod statics;
mod templates;

use crate::cli::{questions, Args, msg};
use crate::templates::{make, get_templates};
use crate::statics::{NOT_IMPLEMENTED};

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
	let notexist = template.contains(&"notimplemented");

	if notexist {
		msg(&format!("{} Option not implemented yet", NOT_IMPLEMENTED));
		return Ok(());
	}
	
	if create_component {
		make::component(
			&answers.name,
			&answers.tool,
			&answers.tool_type,
			&answers.arch_type,
			&answers.path
		)?;
	}

	if create_project {
		make::project(
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
