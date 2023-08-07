use std::option::Option;

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
				change_case(&word.to_lowercase(), Some("capital")).to_string()
			},
		};

		formatted.push(word_formatted);
	}

	formatted.join(&separator)
}
