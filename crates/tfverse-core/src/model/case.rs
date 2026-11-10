//! Word splitting and case conversion for identifiers.

/// A way of joining words into one identifier or phrase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
	/// `MyExample`
	Pascal,
	/// `myExample`
	Camel,
	/// `my-example`
	Kebab,
	/// `my_example`
	Snake,
	/// `MY_EXAMPLE`
	Constant,
	/// `My Example`
	Title,
	/// `my example`
	Lower,
	/// `myexample`
	Flat,
	/// `my/example`, e.g. a folder for the package `my.example`
	Path,
}

impl Case {
	/// Every case, in replacement priority order.
	pub const ALL: [Self; 9] = [
		Self::Pascal,
		Self::Camel,
		Self::Kebab,
		Self::Snake,
		Self::Constant,
		Self::Title,
		Self::Lower,
		Self::Flat,
		Self::Path,
	];

	/// Joins `words` in this case.
	#[must_use]
	pub fn join<S: AsRef<str>>(self, words: &[S]) -> String {
		let words = words.iter().map(AsRef::as_ref);
		match self {
			Self::Pascal => words.map(capitalize).collect(),
			Self::Camel => words
				.enumerate()
				.map(|(index, word)| {
					if index == 0 {
						word.to_lowercase()
					} else {
						capitalize(word)
					}
				})
				.collect(),
			Self::Kebab => lowercase(words).join("-"),
			Self::Snake => lowercase(words).join("_"),
			Self::Constant => words.map(str::to_uppercase).collect::<Vec<_>>().join("_"),
			Self::Title => words.map(capitalize).collect::<Vec<_>>().join(" "),
			Self::Lower => lowercase(words).join(" "),
			Self::Flat => lowercase(words).concat(),
			Self::Path => lowercase(words).join("/"),
		}
	}
}

/// Splits an identifier or phrase into lowercase words.
///
/// `"MyHTTPServer"`, `"my-http-server"` and `"my http server"` all give `["my", "http", "server"]`.
#[must_use]
pub fn words(text: &str) -> Vec<String> {
	let chars: Vec<char> = text.chars().collect();
	let mut words = Vec::new();
	let mut current = String::new();

	for (index, &c) in chars.iter().enumerate() {
		if !c.is_alphanumeric() {
			flush(&mut words, &mut current);
			continue;
		}
		let previous = index.checked_sub(1).map(|i| chars[i]);
		let next = chars.get(index + 1).copied();
		let starts_word = c.is_uppercase()
			&& previous.is_some_and(|p| {
				p.is_lowercase() || p.is_numeric() || (p.is_uppercase() && next.is_some_and(char::is_lowercase))
			});
		if starts_word {
			flush(&mut words, &mut current);
		}
		current.extend(c.to_lowercase());
	}
	flush(&mut words, &mut current);
	words
}

fn flush(words: &mut Vec<String>, current: &mut String) {
	if !current.is_empty() {
		words.push(std::mem::take(current));
	}
}

fn capitalize(word: &str) -> String {
	let mut chars = word.chars();
	chars.next().map_or_else(String::new, |first| {
		first.to_uppercase().chain(chars.flat_map(char::to_lowercase)).collect()
	})
}

fn lowercase<'a>(words: impl Iterator<Item = &'a str>) -> Vec<String> {
	words.map(str::to_lowercase).collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn words_should_split_pascal_case() {
		assert_eq!(words("MyExample"), ["my", "example"]);
	}

	#[test]
	fn words_should_split_acronyms() {
		assert_eq!(words("MyHTTPServer"), ["my", "http", "server"]);
	}

	#[test]
	fn words_should_split_separators() {
		assert_eq!(words("my-http_server name"), ["my", "http", "server", "name"]);
	}

	#[test]
	fn words_should_keep_digits_with_their_word() {
		assert_eq!(words("base64Encoder"), ["base64", "encoder"]);
	}

	#[test]
	fn join_should_produce_every_case() {
		let parts = ["user", "profile"];
		let joined: Vec<String> = Case::ALL.iter().map(|case| case.join(&parts)).collect();

		assert_eq!(
			joined,
			[
				"UserProfile",
				"userProfile",
				"user-profile",
				"user_profile",
				"USER_PROFILE",
				"User Profile",
				"user profile",
				"userprofile",
				"user/profile"
			]
		);
	}
}
