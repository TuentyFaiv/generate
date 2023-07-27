use std::{option::Option, fs::File, io::{BufReader, Read}};

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
	match root {
		Some(path) => match File::open(format!("{path}{template}")) {
			Ok(file) => {
				let mut buf_reader = BufReader::new(&file);
				let mut content = String::new();
				match buf_reader.read_to_string(&mut content) {
					Ok(_) => {},
					Err(_) => return default
				};

				content
			},
			Err(_) => default
		},
		None => default
	}
}
