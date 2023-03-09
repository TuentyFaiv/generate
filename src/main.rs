use std::io::{self, Write, BufRead, BufReader, BufWriter, StdoutLock};
use std::fs::{File, create_dir, create_dir_all};
// use std::path::PathBuf;
use std::collections::HashMap;
use std::process::{Command};
use anyhow::{Context, Result};
use clap::Parser;
// use indicatif::ProgressBar;
use console::{Emoji, style};
use dialoguer::{Select, theme::ColorfulTheme, console::Term, Input, Confirm};

mod cli;
mod statics;
mod templates;

use crate::cli::{choose_option, input, command};
use crate::templates::{svelte, get_templates};
use crate::statics::{TOOLS, TOOLS_REACT, TOOLS_SVELTE, TOOLS_WEBCOMPONENTS, TOOLS_BASE};
use crate::statics::{ARCHS, ARCHS_COMPONENT};
use crate::statics::{NOT_IMPLEMENTED, DONE, OK};

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

fn format_name(name: &String) -> String {
	let splitted: Vec<&str> = name.split(&['-', ' '][..]).collect();
	
	let mut formatted = String::new();

	for word in splitted {
		let mut letters: Vec<char> = word.chars().collect();
		letters[0] = letters[0].to_uppercase().nth(0).unwrap();

		let word_capitalize: String = letters.into_iter().collect();
		
		formatted = format!("{formatted}{word_capitalize}");
	}

	formatted
}


pub fn main() -> Result<()> {

	// env_logger::init();
	let args = Args::parse();

	let tools = TOOLS.to_vec();
	let tools_react = TOOLS_REACT.to_vec();
	let tools_svelte = TOOLS_SVELTE.to_vec();
	let tools_webcomponents = TOOLS_WEBCOMPONENTS.to_vec();
	let tools_base = TOOLS_BASE.to_vec();
	let archs = ARCHS.to_vec();
	let archs_components = ARCHS_COMPONENT.to_vec();
	let templates = get_templates();

	let stdout = io::stdout();
	let mut handle = io::BufWriter::new(stdout.lock());

	let mut name = match args.name.clone() {
		None => {
			input("Proyect name:", "new-proyect")?
		}
		Some(exist) => exist
	};

	name = format_name(&name);

	let path = match args.path.clone() {
		None => {
			input("Location:", format!("./{name}").as_str())?
		}
		Some(exist) => exist
	};

	let tool = match args.tool.clone() {
		None => {
			choose_option("Choose a tool:", tools)?
		}
		Some(exist) => {
			if !tools.contains(&exist.as_str()) {
				choose_option("Choose a tool:", tools)?
			} else {
				exist
			}
		}
	};

	let arch = match args.arch.clone() {
		None => {
			choose_option("Choose an architecture:", archs)?
		}
		Some(exist) => {
			if !archs.contains(&exist.as_str()) {
				choose_option("Choose an architecture:", archs)?
			} else {
				exist
			}
		}
	};

	let is_atomic = arch.as_str() == "atomic";
	let is_library = arch.as_str() == "library";
	let is_component = arch.as_str() == "component";

	let mut arch_type = String::new();

	if is_component {
		arch_type = choose_option("Choose type:", archs_components)?;
	}

	let tool_type = match format!("{tool}-{arch}").as_str() {
		"react-atomic" => {
			choose_option("Chose react project:", [tools_react, tools_base].concat())?
		},
		"svelte-atomic" => {
			choose_option("Chose svelte project:", [tools_svelte, tools_base].concat())?
		},
		"svelte-library" => {
			choose_option("Chose svelte library:", [tools_svelte, tools_webcomponents, tools_base].concat())?
		},
		_ => {
			choose_option("Chose language:", tools_base)?
		}
	};

	let accept = Confirm::with_theme(&ColorfulTheme::default())
		.with_prompt("Are you sure?")
		.default(true)
		.interact()?;

	if !accept {
		return Ok(());
	}

	let template_choosed = format!("{tool}-{tool_type}-{arch}");

	let template = match templates.get(&template_choosed.as_str()) {
		None => {
			[].to_vec()
		},
		Some(&option) => {
			let options: Vec<&str> = option.split('-').collect();

			options
		}
	};

	let create_project = template.contains(&"repo");
	let create_library = template.contains(&"library");
	let create_component = template.contains(&"component");
	let notexist = template.contains(&"notimplemented");

	// println!("{}", create_project);
	// println!("{}", create_library);
	// println!("{}", create_component);

	if notexist {
		println!("");
		println!("{} Option not implemented yet", NOT_IMPLEMENTED);
		println!("");
		return Ok(());
	}
	
	if create_component {
		match tool.as_str() {
			"react" => {
				println!("React component")
			},
			"svelte" => {
				let full_path = match arch_type.as_str() {
					"normal" => {
						format!("./src/ui/{path}/components/{name}")
					},
					_ => {
						format!("./src/ui/{path}/{arch_type}/{name}")
					}
				};

				create_dir_all(&full_path).unwrap_or_else(|why| {
					println!("! {:?}", why.kind());
				});

				svelte::generate(&full_path.as_str(), &name.as_str(), &tool_type)?;
			},
			"vanilla" => {
				println!("Vanilla component")
			},
			_ => {}
		}
	}

	if create_project {
		let repository = *template.get(1).unwrap();
		let url = format!("git@github.com:Platimex/{repository}.git");
		let commit = format!("🎉 FEAT: Starting project {name}");

		command("git", ["clone", url.as_str(), path.as_str()].to_vec(), None, Some(format!("Failed to generate {tool} with {arch}").as_str()));		

		// if create_library {
		// 	command("git", ["switch", "library"].to_vec(), Some(&path.as_str()), Some("Failed to switch to library"));
		// }

		command("rm", ["-rf", ".git"].to_vec(), Some(&path.as_str()), Some("Failed to reset git"));

		command("git", ["init", "-b", "main"].to_vec(), Some(&path.as_str()), Some("Failed to restart git"));

		command("git", ["add", "."].to_vec(), Some(&path.as_str()), Some("Failed to staging files"));

		command("git", ["commit", "-m", commit.as_str(), "-m", "\"\"", "--no-gpg-sign"].to_vec(), Some(&path.as_str()), Some("Failed to commit"));

		command("git", ["remote", "add", "template", url.as_str()].to_vec(), Some(&path.as_str()), Some("Failed to add remote repository"));
	}

	println!("");
	println!("{} {}", DONE, style("All done").cyan());
	println!("");
	if is_component {
		println!("{} {}", OK, style(format!("Component created at {path}")).cyan());
	} else {
		println!("{}", style(format!("Mote to {path} and start a new universe")).cyan());
	}
	println!("");

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
