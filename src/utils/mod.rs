use std::option::Option;

pub fn change_case(word: &str, to: Option<&str>) -> String {
  let mut letters: Vec<char> = word.chars().collect();
  letters[0] = match to {
		Some("capital") => letters[0].to_uppercase().nth(0).unwrap(),
		Some("camel") => letters[0].to_lowercase().nth(0).unwrap(),
		_ => letters[0].to_uppercase().nth(0).unwrap(),
	};

  let word_capitalize: String = letters.into_iter().collect();

  word_capitalize
}

pub fn transform(name: &String, to: Option<&str>) -> String {
	let splitted: Vec<&str> = name.split(&['-', '_', ' '][..]).collect();
	let separator = match to {
		Some("dash") => "_",
		Some("text") => "",
		_ => "",
	};
	
	let mut formatted: Vec<String> = Vec::new();

	for word in splitted {
		let word_formatted = match to {
			Some("lower") => word.to_lowercase(),
			Some("dash") => change_case(word, None),
			Some("text") => change_case(word, None),
			_ => change_case(word, None),
		};

		formatted.push(word_formatted);
	}

	formatted.join(&separator)
}
