pub mod questions;
pub mod actions;
pub mod enums;
pub mod structs;

use std::io::{self, Write};
use std::process::Command;

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Select, Input};
use dialoguer::{console::Term, theme::ColorfulTheme};

use crate::statics::DONE;

pub fn sure() -> Result<bool> {
	let accept = Confirm::with_theme(&ColorfulTheme::default())
		.with_prompt("Are you sure?")
		.default(true)
		.wait_for_newline(true)
		.interact()?;
	Ok(accept)
}

pub fn arg_or(prompt: &str, arg: Option<String>, options: &Vec<String>) -> Result<String> {
	let value = match arg {
		None => choose_option(prompt, options)?,
		Some(exist) => {
			if !options.contains(&exist) {
				choose_option(prompt, options)?
			} else {
				exist
			}
		}
	};

	Ok(value)
}

pub fn choose_option(prompt: &str, options: &Vec<String>) -> Result<String> {
	let selection = Select::with_theme(&ColorfulTheme::default())
		.with_prompt(prompt)
		.items(&options)
		.default(0)
		.interact_on_opt(&Term::stdout())?;

	let option = match selection {
		Some(index) => options[index].clone(),
		None => "Not valid option".to_string(),
	};

	Ok(option)
}

pub fn input(prompt: &str, default: &str) -> Result<String> {
	let value = Input::<String>::with_theme(&ColorfulTheme::default())
    .with_prompt(prompt)
    .default(default.into())
    .interact_text_on(&Term::stdout())?;

	Ok(value)
}

pub fn command(program: &str, args: Vec<&str>, path: Option<&str>, error_msg: Option<&str>) {
	let mut cmd = Command::new(program);
	cmd
		.current_dir(path.unwrap_or("./"))
		.args(args)
		.output()
		.expect(error_msg.unwrap_or("Failed to execute command"));
}

pub fn msg(content: &String) {
	let stdout = io::stdout();
	let mut handle = io::BufWriter::new(stdout.lock());

	writeln!(handle, "").unwrap();
	writeln!(handle, "{}", content).unwrap();
}

pub fn done() {
	msg(&format!("{} {}", DONE, style("All done").cyan()));
}