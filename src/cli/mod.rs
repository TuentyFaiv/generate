use std::process::Command;
use anyhow::{Result};
use dialoguer::{Select, console::Term, theme::ColorfulTheme, Input};

pub fn choose_option(prompt: &str, options: Vec<&str>) -> Result<String> {
	let selection = Select::with_theme(&ColorfulTheme::default())
		.with_prompt(prompt)
		.items(&options)
		.default(0)
		.interact_on_opt(&Term::stderr())?;

	let option = match selection {
		Some(index) => {
			options[index].to_string()
		},
		None => "Not valid option".to_string()

	};

	Ok(option)
}

pub fn input(prompt: &str, default: &str) -> Result<String> {
	let value = Input::with_theme(&ColorfulTheme::default())
    .with_prompt(prompt)
    .default(default.into())
    .interact_text()?;

	Ok(value)
}

pub fn command(program: &str, args: Vec<&str>, path: Option<&str>, error_msg: Option<&str>) {
	let mut cmd = Command::new(program);
	cmd
		.current_dir(path.unwrap_or("."))
		.args(args)
		.output()
		.expect(error_msg.unwrap_or("Failed to execute command"));

	// println!("Out: {:?}", cmd.output());
}
