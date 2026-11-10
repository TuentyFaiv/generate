//! Printing results for people, or as JSON for tools and agents.

use std::fmt::Write as _;
use std::io::IsTerminal;

use serde::Serialize;
use tfverse_core::history::Transaction;
use tfverse_core::plan::{Action, Plan, Rejected, Validated};

/// How results are printed.
#[derive(Clone, Copy, Debug)]
pub struct Output {
	/// Print diffs of changed files.
	pub diff: bool,
	color: bool,
}

impl Output {
	/// Text with color on a terminal unless `NO_COLOR` is set, or JSON.
	pub fn new(json: bool, diff: bool) -> Self {
		Self {
			diff,
			color: !json && std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
		}
	}

	/// Prints `value` as pretty JSON.
	pub fn json(value: &impl Serialize) {
		println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
	}

	fn paint(self, code: &str, text: &str) -> String {
		if self.color {
			format!("\x1b[{code}m{text}\x1b[0m")
		} else {
			text.to_owned()
		}
	}

	/// A plan's summary, changes, packages and notes.
	pub fn plan(self, plan: &Plan<Validated>) -> String {
		let mut text = format!("{}\n", self.paint("1", plan.summary()));
		let changed: Vec<_> = plan
			.changes()
			.iter()
			.filter(|change| change.action != Action::Unchanged)
			.collect();
		if changed.is_empty() {
			text.push_str("  nothing to change\n");
		}
		for change in &changed {
			let (label, code) = match change.action {
				Action::Create => ("create", "32"),
				Action::Modify => ("modify", "33"),
				Action::Overwrite => ("replace", "31"),
				Action::Delete => ("delete", "31"),
				_ => ("same", "2"),
			};
			let _ = write!(text, "  {} {}", self.paint(code, &format!("{label:<7}")), change.path);
			if change.conflicts > 0 {
				let _ = write!(
					text,
					"  {}",
					self.paint("31", &format!("{} conflict(s)", change.conflicts))
				);
			}
			if let Some(risk) = change.risk {
				let _ = write!(text, "  {}", self.paint("35", &format!("({risk})")));
			}
			text.push('\n');
			if self.diff
				&& let Some(diff) = &change.diff
			{
				for line in diff.lines() {
					let code = match line.chars().next() {
						Some('+') => "32",
						Some('-') => "31",
						Some('@') => "36",
						_ => "0",
					};
					let _ = writeln!(text, "      {}", self.paint(code, line));
				}
			}
		}
		if !plan.dependencies().is_empty() {
			text.push_str("Packages to add (tfverse does not install them):\n");
			for dependency in plan.dependencies() {
				let dev = if dependency.dev { " (dev)" } else { "" };
				let _ = writeln!(
					text,
					"  {:<8} {} {}{dev}",
					format!("{:?}", dependency.ecosystem).to_lowercase(),
					dependency.name,
					dependency.version
				);
			}
		}
		if !plan.notes().is_empty() {
			text.push_str("Notes:\n");
			for note in plan.notes() {
				let _ = writeln!(text, "  {}: {}", note.path, note.message);
			}
		}
		text
	}

	/// A rejected plan's problems.
	pub fn rejected(self, rejected: &Rejected) -> String {
		let mut text = format!("{}\n", self.paint("31;1", "The plan was rejected:"));
		for problem in &rejected.problems {
			let _ = writeln!(text, "  {}: {}", problem.path, problem.message);
		}
		text
	}

	/// What an apply did.
	pub fn applied(self, transaction: &Transaction) -> String {
		format!(
			"{} transaction {} ({} file(s)). Undo it with `tfverse undo`.\n",
			self.paint("32;1", "Applied"),
			transaction.id,
			transaction.changes.len()
		)
	}
}
