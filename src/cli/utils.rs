use std::io::{self, Write, Error};
use std::fs::read_dir;
use std::process::Command;

use anyhow::Result;
use console::style;
use dialoguer::{Confirm, Select, Input};
use dialoguer::{console::Term, theme::ColorfulTheme};

use crate::statics::DONE;
use crate::utils::transform;

pub fn sure() -> Result<bool> {
	let accept = Confirm::with_theme(&ColorfulTheme::default())
		.with_prompt("Are you sure?")
		.default(true)
		.wait_for_newline(true)
		.interact()?;
	Ok(accept)
}

pub fn list_folders(path_root: &str) -> Result<Vec<String>> {
	let slash = if cfg!(target_os = "windows") { "\\" } else { "/" };
	let entries = match read_dir(path_root) {
		Ok(paths) => paths.map(|entry| entry.map(|entry| entry.path()))
			.map(|path| path.map(|path_ns| {
				path_ns.to_string_lossy().to_string().replace(&format!("{path_root}{slash}"), "")
			}))
			.collect::<Result<Vec<_>, Error>>()?,
		Err(_) => ["sharing".to_owned()].to_vec()
	};

	Ok(entries)
}

pub fn show_namespaces(paths: &Vec<&String>) -> Result<String> {
	let mut options = paths.iter().map(|path| {
		list_folders(*path).unwrap()
	}).collect::<Vec<Vec<String>>>().concat();
	options.sort();
	options.dedup();
	let namespace = choose_option("Namespace:", &[options, ["custom".to_owned()].to_vec()].concat())?;

	if namespace.as_str() == "custom" {
		let namespace = input("New namespace:", "namespace")?;
		Ok(transform(&namespace, Some("lower")))
	} else {
		Ok(namespace)
	}
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