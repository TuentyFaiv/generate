use std::{option::Option};

use crate::config::file::{RepositoryTool, ConfigRepositoryTool};

pub fn to_vec(arr: &[&str]) -> Vec<String> {
	arr.iter().map(|&s| s.to_string()).collect::<Vec<String>>()
}

pub fn to_tool_type(tools: &[(&str, Option<RepositoryTool>)]) -> Option<Vec<(String, Option<ConfigRepositoryTool>)>> {
	Some(tools.iter().map(|(name, repository)| {
		(name.to_string(), match repository {
			None => None,
			Some(repository) => Some(ConfigRepositoryTool {
				name: Some(repository.name.to_string()),
				project: Some(repository.project.to_string()),
				library: match repository.library {
					None => None,
					Some(library) => Some(library.to_string()),
				},
			}),
		})
	}).collect::<Vec<(String, Option<ConfigRepositoryTool>)>>())
}

pub fn tool_to_vec(tools: &[(&str, Option<RepositoryTool>)]) -> Vec<String> {
	tools.iter().map(|(tool, _)| {
		tool.to_string()
	}).collect()
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
