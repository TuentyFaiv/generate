//! Reviewing a plan hunk by hunk at the terminal, like `git add -p`.

use std::collections::BTreeSet;

use tfverse_core::plan::{Action, Pick, Plan, Selection, Validated};

use super::prompt::ask;

/// Asks about every hunk of every change and narrows `plan` to the answers.
pub fn review(plan: &mut Plan<Validated>) -> anyhow::Result<()> {
	let mut selection = Selection::new();
	let mut quit = false;
	for change in plan.changes() {
		if change.action == Action::Unchanged {
			continue;
		}
		if quit {
			selection.insert(change.path.clone(), Pick::None);
			continue;
		}
		let hunks = change.hunks();
		let mut keep = BTreeSet::new();
		let mut rest: Option<bool> = None;
		eprintln!("\n{} {}", format!("{:?}", change.action).to_lowercase(), change.path);
		for hunk in &hunks {
			let answer = if let Some(answer) = rest {
				answer
			} else {
				eprintln!("{}\n{}", hunk.header, hunk.lines.trim_end());
				loop {
					match ask("Keep this hunk? [y]es, [n]o, [a]ll in file, [d]rop rest of file, [q]uit: ")?.as_str() {
						"y" | "yes" => break true,
						"n" | "no" => break false,
						"a" => {
							rest = Some(true);
							break true;
						}
						"d" => {
							rest = Some(false);
							break false;
						}
						"q" => {
							quit = true;
							rest = Some(false);
							break false;
						}
						_ => {}
					}
				}
			};
			if answer {
				keep.insert(hunk.index);
			}
		}
		let pick = if keep.len() == hunks.len() {
			Pick::All
		} else if keep.is_empty() {
			Pick::None
		} else {
			Pick::Hunks(keep)
		};
		selection.insert(change.path.clone(), pick);
	}
	plan.select(&selection)?;
	Ok(())
}
