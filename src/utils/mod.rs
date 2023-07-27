use std::{option::Option, fs::File, io::{BufReader, Read}};
use dirs_next::home_dir;

pub fn to_vec(arr: &[&str]) -> Vec<String> {
	arr.iter().map(|&s| s.to_string()).collect::<Vec<String>>()
}

pub fn change_case(word: &str, to: Option<&str>) -> String {
  let mut letters: Vec<char> = word.chars().collect();

  letters[0] = match to {
		Some("camel") => letters[0].to_lowercase().nth(0).unwrap(),
		Some("capital") | _ => letters[0].to_uppercase().nth(0).unwrap(),
	};

  let word_capitalize: String = letters.into_iter().collect();

	word_capitalize
}

pub fn transform(name: &str, to: Option<&str>) -> String {
	let splitted: Vec<&str> = name.split(&['-', '_', ' '][..]).collect();
	let separator = match to {
		Some("dash") => "_",
		Some("text") | _ => "",
	};
	
	let mut formatted: Vec<String> = Vec::new();

	for word in splitted {
		let word_formatted = match to {
			Some("lower") => word.to_lowercase(),
			Some("dash") | Some("text") | _ => {
				change_case(word, Some("capital")).to_string()
			},
		};

		formatted.push(word_formatted);
	}

	formatted.join(&separator)
}

pub fn read_path(root: &Option<String>, template: String, default: String) -> String {
	println!("Template dir: {}\nTemplate file: {template}", root.clone().unwrap_or("".to_string()));

	match home_dir() {
		None => default,
		Some(mut home) => match root {
			Some(path) => {
				let template_path = format!("{path}{template}");
				home.push(template_path.as_str());

				println!("Template path: {:?}", home.to_str());
				match home.to_str() {
					None => default,
					Some(full_path) => match File::open(full_path) {
						Ok(file) => {
							let mut buf_reader = BufReader::new(&file);
							let mut content = String::new();
							match buf_reader.read_to_string(&mut content) {
								Ok(readed) => {
									println!("{readed}");
								},
								Err(error) => {
									println!("{error}");
									return default
								}
							};

							println!("Content: {content}");

							content
						},
						Err(error) => {
							println!("{error}");

							default
						}
					}
				}
			},
			None => default
		}
	}
}
