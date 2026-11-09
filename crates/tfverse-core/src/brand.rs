//! Every user-visible name, derived from [`brand_name!`](crate::brand_name).

/// Expands to the product name as a string literal.
#[macro_export]
macro_rules! brand_name {
	() => {
		"tfverse"
	};
}

/// Product name as shown to users.
pub const NAME: &str = brand_name!();

/// Executable name, without the `.exe` suffix.
pub const BINARY_NAME: &str = brand_name!();

/// JSON project config file.
pub const CONFIG_FILE_JSON: &str = concat!(brand_name!(), ".json");

/// TOML project config file.
pub const CONFIG_FILE_TOML: &str = concat!(brand_name!(), ".toml");

/// Accepted project config files; a project may use only one.
pub const CONFIG_FILES: [&str; 2] = [CONFIG_FILE_JSON, CONFIG_FILE_TOML];

/// Lockfile with installed packs, hashes and per-file base versions.
pub const LOCK_FILE: &str = concat!(brand_name!(), ".lock");

/// Prefix for every environment variable, e.g. `TFVERSE_LOG`.
pub const ENV_PREFIX: &str = "TFVERSE_";

/// npm scope for the wrapper and platform packages.
pub const NPM_SCOPE: &str = "@tuentyfaiv";

/// npm package users install.
pub const NPM_PACKAGE: &str = "@tuentyfaiv/cli";

/// Public documentation site.
pub const DOCS_URL: &str = "https://cli.tuentyfaiv.com";

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn env_prefix_should_match_upper_case_name() {
		let expected = format!("{}_", NAME.to_uppercase());

		assert_eq!(ENV_PREFIX, expected);
	}

	#[test]
	fn npm_package_should_live_in_npm_scope() {
		let prefix = format!("{NPM_SCOPE}/");

		assert!(
			NPM_PACKAGE.starts_with(&prefix),
			"{NPM_PACKAGE} is not inside {NPM_SCOPE}"
		);
	}

	#[test]
	fn config_files_should_be_distinct() {
		assert_ne!(CONFIG_FILE_JSON, CONFIG_FILE_TOML);
	}

	#[test]
	fn config_files_should_start_with_name() {
		assert!(
			CONFIG_FILES.iter().all(|file| file.starts_with(NAME)),
			"{CONFIG_FILES:?}"
		);
	}
}
