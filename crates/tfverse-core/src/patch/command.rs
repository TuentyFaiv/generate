//! A user-configured external patcher speaking JSON over stdin and stdout.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::patch::Patch;
use crate::patch::{PatchError, Patched, Patcher};
use crate::process;

const TIMEOUT: Duration = Duration::from_secs(30);

/// Runs an external program for one patch kind.
#[derive(Clone, Debug)]
pub struct CommandPatcher {
	name: String,
	program: String,
	args: Vec<String>,
}

#[derive(Serialize)]
struct Request<'a> {
	path: &'a str,
	source: &'a str,
	patch: &'a Patch,
}

#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Response {
	Changed { source: String, inserted: String },
	Unchanged { inserted: String },
}

impl CommandPatcher {
	/// A patcher running `command[0]` with the remaining arguments.
	pub fn new(name: &str, command: &[String]) -> Result<Self, PatchError> {
		let (program, args) = command.split_first().ok_or_else(|| PatchError::External {
			name: name.to_owned(),
			message: "command is empty".to_owned(),
		})?;
		Ok(Self {
			name: name.to_owned(),
			program: program.clone(),
			args: args.to_vec(),
		})
	}

	fn fail(&self, message: impl Into<String>) -> PatchError {
		PatchError::External {
			name: self.name.clone(),
			message: message.into(),
		}
	}
}

impl Patcher for CommandPatcher {
	fn apply(&self, text: &str, patch: &Patch) -> Result<Patched, PatchError> {
		let request = serde_json::to_vec(&Request {
			path: patch.file().as_str(),
			source: text,
			patch,
		})
		.map_err(|error| self.fail(error.to_string()))?;
		let output =
			process::run(&self.program, &self.args, request, TIMEOUT).map_err(|error| self.fail(error.to_string()))?;
		if !output.status.success() {
			return Err(self.fail(format!("exited with {}", output.status)));
		}
		let output = output.stdout;

		match serde_json::from_slice(&output).map_err(|error| self.fail(format!("invalid response: {error}")))? {
			Response::Changed { source, inserted } => Ok(Patched::Changed { text: source, inserted }),
			Response::Unchanged { inserted } => Ok(Patched::AlreadyApplied { inserted }),
		}
	}
}

#[cfg(all(test, unix))]
mod tests {
	use super::*;
	use crate::model::RelPath;

	fn patch() -> Patch {
		Patch::Import {
			file: RelPath::constant("a.ts"),
			import: "import a from 'a';".to_owned(),
		}
	}

	#[test]
	fn should_use_the_tool_response() {
		let script = r#"cat > /dev/null; printf '{"status":"changed","source":"patched","inserted":"x"}'"#;
		let patcher = CommandPatcher::new("sh", &["sh".to_owned(), "-c".to_owned(), script.to_owned()]).unwrap();

		assert_eq!(
			patcher.apply("", &patch()),
			Ok(Patched::Changed {
				text: "patched".to_owned(),
				inserted: "x".to_owned()
			})
		);
	}

	#[test]
	fn should_fail_on_non_zero_exit() {
		let patcher = CommandPatcher::new("sh", &["sh".to_owned(), "-c".to_owned(), "exit 3".to_owned()]).unwrap();

		assert!(matches!(patcher.apply("", &patch()), Err(PatchError::External { .. })));
	}

	#[test]
	fn should_reject_empty_command() {
		assert!(CommandPatcher::new("empty", &[]).is_err());
	}
}
