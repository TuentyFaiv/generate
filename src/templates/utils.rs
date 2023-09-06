use std::fs::File;
use std::io::{BufReader, Read};
use std::option::Option;
use dirs_next::home_dir;

use crate::cli::structs::AnswersName;
use crate::create::structs::CreationPaths;

use super::constants::{NAMESPACE, NAME_CAMEL, NAME_PASCAL, NAME_DASH, NAME_CONSTANT, NAME_SNAKE, NAME_LOWER, NAME};

pub fn read_path(root: &Option<String>, template: &CreationPaths) -> String {
	let CreationPaths { default, template } = template;
	let default = default.to_owned();
	// println!("Template dir: {}\nTemplate file: {template}", root.clone().unwrap_or("".to_string()));

	match home_dir() {
		None => default,
		Some(mut home) => match root {
			Some(path) => {
				let template_path = format!("{path}{template}");
				home.push(template_path.as_str());

				// println!("Template path: {:?}", home.to_str());
				match home.to_str() {
					None => default,
					Some(full_path) => match File::open(full_path) {
						Ok(file) => {
							let mut buf_reader = BufReader::new(&file);
							let mut content = String::new();
							match buf_reader.read_to_string(&mut content) {
								Ok(_) => {
									// println!("{readed}");
								},
								Err(_) => {
									// println!("{error}");
									return default
								}
							};

							// println!("Content: {content}");

							content
						},
						Err(_) => {
							// println!("{error}");

							default
						}
					}
				}
			},
			None => default
		}
	}
}

pub fn set_keywords(content: &String, name: &AnswersName) -> String {
	let mut content = content.clone();

	content = content.replace(NAMESPACE, &name.namespace);
	content = content.replace(NAME_CAMEL, &name.camel);
	content = content.replace(NAME_PASCAL, &name.pascal);
	content = content.replace(NAME_DASH, &name.dash);
	content = content.replace(NAME_CONSTANT, &name.constant);
	content = content.replace(NAME_SNAKE, &name.snake);
	content = content.replace(NAME_LOWER, &name.lower);
	content = content.replace(NAME, &name.original);

	content
}