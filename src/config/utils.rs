use super::file::{RepositoryTool, ConfigRepositoryTool};

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


pub fn default_tool(tools: &Option<Vec<(String, Option<ConfigRepositoryTool>)>>, default: Vec<String>) -> Vec<String> {
 	match &tools {
		Some(has_tool) => {
			has_tool.iter().map(|(tool, _)| tool.clone()).collect()
		},
		None => default
	}
}

pub fn list_projects(tools: &Option<Vec<(String, Option<ConfigRepositoryTool>)>>) -> Vec<String> {
	match &tools {
		None => [].to_vec(),
		Some(tool) => tool.iter().filter_map(|(name, repository)| -> Option<String> {
			match repository {
				Some(repo) => match repo.project {
					Some(_) => Some(name.clone()),
					None => None
				},
				None => None
			}
		}).collect()
	}
}

pub fn list_libraries(tools: &Option<Vec<(String, Option<ConfigRepositoryTool>)>>) -> Vec<String> {
	match &tools {
		None => [].to_vec(),
		Some(tool) => tool.iter().filter_map(|(name, repository)| -> Option<String> {
			match repository {
				Some(repo) => match repo.library {
					Some(_) => Some(name.clone()),
					None => None
				},
				None => None
			}
		}).collect()
	}
}

pub fn search_repository(tools: &Option<Vec<(String, Option<ConfigRepositoryTool>)>>, search: &str) -> Option<ConfigRepositoryTool> {
	match &tools {
		None => None,
		Some(tool) => match tool.iter().find(|(name, _)| name.as_str() == search) {
			None => None,
			Some(tool) => tool.1.clone()
		}
	}
}