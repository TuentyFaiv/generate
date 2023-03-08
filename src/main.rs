use std::io::{self, Write, BufRead, BufReader, BufWriter, StdoutLock};
use std::fs::{File, create_dir};
// use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;
use anyhow::{Context, Result};
// use indicatif::ProgressBar;
use clap::Parser;
use dialoguer::{Select, theme::ColorfulTheme, console::Term, Input, Confirm};

// use log::info;

const TOOLS: &'static [&'static str] = &["react-ts", "react", "svelte-ts", "svelte", "typescript", "javascript"];
const ARCHS: &'static [&'static str] = &["atomic", "library", "component", "container"];

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
	/// Project name
	name: Option<String>,
	/// Tool to generate template
	#[arg(short, long)]
	tool: Option<String>,
	/// Frontend architecture
  #[arg(short, long)]
	arch: Option<String>,
	/// Path to generate template
  #[arg(short, long)]
	path: Option<String>
}

fn my_print(handle: &mut BufWriter<StdoutLock>, text: &String) {
	writeln!(handle, "{:?}", text).unwrap();
}

fn choose_option(prompt: &str, options: Vec<&str>) -> Result<String> {
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

fn input(prompt: &str, default: &str) -> Result<String> {
	let value = Input::with_theme(&ColorfulTheme::default())
    .with_prompt(prompt)
    .default(default.into())
    .interact_text()?;

	Ok(value)
}

fn main() -> Result<()> {
	// env_logger::init();
	let args = Args::parse();

	let tools = TOOLS.to_vec();
	let archs = ARCHS.to_vec();

	let stdout = io::stdout();
	let mut handle = io::BufWriter::new(stdout.lock());

	let name = match args.name.clone() {
		None => {
			input("Proyect name", "new-proyect")?
		}
		Some(exist) => exist
	};

	let path = match args.path.clone() {
		None => {
			input("Proyect location", format!("./{name}").as_str())?
		}
		Some(exist) => exist
	};

	let tool = match args.tool.clone() {
		None => {
			choose_option("Choose a tool", tools)?
		}
		Some(exist) => {
			if !tools.contains(&exist.as_str()) {
				choose_option("Choose a tool", tools)?
			} else {
				exist
			}
		}
	};

	let arch = match args.arch.clone() {
		None => {
			choose_option("Choose an architecture", archs)?
		}
		Some(exist) => {
			if !archs.contains(&exist.as_str()) {
				choose_option("Choose an architecture", archs)?
			} else {
				exist
			}
		}
	};

	if !Confirm::with_theme(&ColorfulTheme::default())
		.with_prompt("Are you sure?")
		.default(true)
		.interact()? {
		return Ok(());
	}

	let mut templates = HashMap::<String, String>::new();
	templates.insert("react-ts-atomic".to_string(), "DittoReact".to_string());
	templates.insert("svelte-ts-atomic".to_string(), "DittoSvelte".to_string());

	let template = format!("{tool}-{arch}");

	match templates.get(&template) {
		None => {
			my_print(&mut handle, &"repository not found".to_string());
		},
		Some(repository) => {
			Command::new("git")
				.args(["clone", format!("git@github.com:Platimex/{repository}.git").as_str(), path.as_str()])
				.output()
				.expect(format!("Failed to generate {tool} with {arch}").as_str());

			// let pwd = Command::new("pwd")
			// 	.output()
			// 	.expect("Not exist");

			// println!("{:?}", pwd);
			// let cd = Command::new("cd")
			// 	.current_dir("./")
			// 	.arg(path.as_str())
			// 	.spawn()
			// 	.expect(format!("Failed to cd to {path}").as_str());

			// println!("{:?}", cd);
		
			// Command::new("rm")
			// 	.args(["-rf", ".git"])
			// 	.output()
			// 	.expect("Failed to reset git");

			// Command::new("git")
			// 	.args(["remote", "add", "template", format!("git@github.com:Platimex/{repository}.git").as_str()])
			// 	.output()
			// 	.expect("Failed to add remote repository");
		}
	};

	// my_print(&mut handle, &name);
	// my_print(&mut handle, &path);
	// my_print(&mut handle, &tool);
	// my_print(&mut handle, &arch);

	// create_dir(path)?;

	// info!("Reading file");
	// let file = File::open(&args.path)
	// 	.with_context(|| format!("Could not read file `{}`", path))?;
	// let content = BufReader::new(file);

	// let stdout = io::stdout();
	// let mut handle = io::BufWriter::new(stdout.lock());

	// for line in content.lines() {
	// 	let text = &line?;

	// 	if text.contains(&args.pattern) {
	// 		// println!("{:?}", text);
	// 		writeln!(handle, "{:?}", text)?;
	// 	}
	// }

	// let pb = ProgressBar::new(100);

	// for i in 0..100 {
	// 	pb.println(format!("[+] finished #{}", i));
	// 	pb.inc(1);
	// }

	// pb.finish_with_message("Done");

	Ok(())
}
