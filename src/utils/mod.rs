pub fn capitalize(word: &str) -> String {
  let mut letters: Vec<char> = word.chars().collect();
  letters[0] = letters[0].to_uppercase().nth(0).unwrap();

  let word_capitalize: String = letters.into_iter().collect();

  word_capitalize
}

pub fn camel(word: &str) -> String {
  let mut letters: Vec<char> = word.chars().collect();
  letters[0] = letters[0].to_lowercase().nth(0).unwrap();

  let word_capitalize: String = letters.into_iter().collect();

  word_capitalize
}

pub fn format_text(name: &String) -> String {
	let splitted: Vec<&str> = name.split(&['-', '_', ' '][..]).collect();
	
	let mut formatted = String::new();

	for word in splitted {
		let word_capitalize = capitalize(word);
		
		formatted = format!("{formatted}{word_capitalize}");
	}

	formatted
}

pub fn format_lower(name: &String) -> String {
	let splitted: Vec<&str> = name.split(&['-', '_', ' '][..]).collect();
	
	let mut formatted = String::new();

	for word in splitted {
		let word_capitalize = word.to_lowercase();
		
		formatted = format!("{formatted}{word_capitalize}");
	}

	formatted
}

pub fn format_dash(name: &String) -> String {
	let splitted: Vec<&str> = name.split(&['-', '_', ' '][..]).collect();
	
	let mut formatted = String::new();

	for word in splitted {
		let word_capitalize = capitalize(word);
		
		formatted = format!("{formatted}_{word_capitalize}");
	}

	formatted
}
