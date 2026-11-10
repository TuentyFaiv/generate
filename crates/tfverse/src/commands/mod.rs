//! The commands, each a method of [`Session`]; plans all end in [`Session::finish`].

mod info;
mod pack;
mod piece;
mod update;

use anyhow::bail;
use serde_json::json;
use tfverse_core::plan::{Action, Plan, Validated};

use crate::cli::{ApplyArgs, Command};
use crate::context::Context;
use crate::ui::{Output, confirm, review};
use crate::{Exit, USAGE};

/// How the current command talks to the user.
#[derive(Clone, Copy)]
pub struct Session {
	/// Print JSON instead of text.
	pub json: bool,
	/// Prompts and confirmations are possible.
	pub interactive: bool,
}

impl Session {
	/// Runs `command` in the project at `context`.
	pub fn run(self, context: &mut Context, command: &Command) -> anyhow::Result<()> {
		match command {
			Command::Add(args) => self.piece(context, args, false),
			Command::Adopt(args) => self.piece(context, args, true),
			Command::Update(args) => self.update(context, args),
			Command::Status => self.status(context),
			Command::Why { location } => self.why(context, location),
			Command::Log => self.log(context),
			Command::Undo { force } => self.undo(context, *force),
			Command::Detect => self.detect(context),
			Command::Pack(command) => self.pack(context, command),
			Command::Mcp | Command::Rpc => Ok(()),
		}
	}

	/// Shows `plan`, lets the user review it, and applies it when they agree.
	pub(super) fn finish(self, context: &Context, mut plan: Plan<Validated>, args: &ApplyArgs) -> anyhow::Result<()> {
		let output = Output::new(self.json, args.diff);
		if args.review {
			if !self.interactive {
				bail!("--review needs a terminal");
			}
			review(&mut plan)?;
		}
		if self.json && args.dry_run {
			Output::json(&json!({ "plan": plan.report(), "applied": null }));
			return Ok(());
		}
		if !self.json {
			print!("{}", output.plan(&plan));
		}
		if args.dry_run {
			return Ok(());
		}
		let nothing = plan.changes().iter().all(|change| change.action == Action::Unchanged) && plan.instances().is_empty();
		if nothing {
			if self.json {
				Output::json(&json!({ "plan": plan.report(), "applied": null }));
			}
			return Ok(());
		}
		let approved = args.yes || (self.interactive && confirm("Apply these changes?")?);
		if !approved {
			if self.json {
				Output::json(&json!({ "plan": plan.report(), "applied": null, "error": "pass --yes to apply" }));
			} else if self.interactive {
				println!("Nothing was changed.");
				return Ok(());
			} else {
				eprintln!("Nothing was changed; pass --yes to apply without asking, or --dry-run to only plan.");
			}
			return Err(Exit(USAGE).into());
		}
		let report = serde_json::to_value(plan.report())?;
		let transaction = context.engine.apply(plan)?;
		if self.json {
			Output::json(&json!({ "plan": report, "applied": transaction }));
		} else {
			print!("{}", output.applied(&transaction));
		}
		Ok(())
	}
}
