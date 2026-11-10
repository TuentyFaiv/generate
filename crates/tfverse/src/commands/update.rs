//! `update`: move everything from a pack to another version, merging the user's edits.

use anyhow::{anyhow, bail};
use tfverse_core::engine::UpdateRequest;

use super::Session;
use crate::cli::UpdateArgs;
use crate::context::Context;

impl Session {
	pub(super) fn update(self, context: &mut Context, args: &UpdateArgs) -> anyhow::Result<()> {
		let fresh = args
			.from
			.as_deref()
			.map(|source| context.load(source, false))
			.transpose()?;
		let pack = match &fresh {
			Some(pack) if pack.manifest().name.as_str() != args.pack => {
				bail!(
					"`{}` is pack `{}`, not `{}`",
					args.from.as_deref().unwrap_or_default(),
					pack.manifest().name,
					args.pack
				)
			}
			Some(pack) => pack,
			None => context
				.library
				.get(&args.pack)
				.ok_or_else(|| anyhow!("no pack `{}` is loaded; use --from <source>", args.pack))?,
		};
		let plan = context.engine.plan_update(
			&UpdateRequest {
				pack,
				granted: None,
				on_conflict: args.on_conflict.into(),
				force: args.apply.force,
				migrate: !args.no_migrate,
			},
			&context.library,
		)?;
		self.finish(context, plan, &args.apply)
	}
}
