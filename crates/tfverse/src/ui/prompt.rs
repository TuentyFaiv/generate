//! Questions asked at the terminal, written to stderr so stdout stays clean for results.

use std::io::{self, BufRead, Write};

/// One line typed at the terminal, trimmed.
pub fn ask(question: &str) -> anyhow::Result<String> {
	let mut stderr = io::stderr();
	write!(stderr, "{question}")?;
	stderr.flush()?;
	let mut answer = String::new();
	io::stdin().lock().read_line(&mut answer)?;
	Ok(answer.trim().to_owned())
}

/// Whether the user answered yes to `question`.
pub fn confirm(question: &str) -> anyhow::Result<bool> {
	Ok(matches!(
		ask(&format!("{question} [y/N] "))?.to_ascii_lowercase().as_str(),
		"y" | "yes"
	))
}
