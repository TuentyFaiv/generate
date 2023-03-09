pub fn capitalize(word: &str) -> String {
  let mut letters: Vec<char> = word.chars().collect();
  letters[0] = letters[0].to_uppercase().nth(0).unwrap();

  let word_capitalize: String = letters.into_iter().collect();

  word_capitalize
}

pub fn format_name(name: &String) -> String {
	let splitted: Vec<&str> = name.split(&['-', ' '][..]).collect();
	
	let mut formatted = String::new();

	for word in splitted {
		let word_capitalize = capitalize(word);
		
		formatted = format!("{formatted}{word_capitalize}");
	}

	formatted
}