//! `add` and `adopt`: plan a piece from its pack and inputs.

use anyhow::anyhow;
use tfverse_core::PieceRequest;

use super::Session;
use crate::cli::PieceArgs;
use crate::context::Context;
use crate::inputs;

impl Session {
	pub(super) fn piece(self, context: &mut Context, args: &PieceArgs, adopt: bool) -> anyhow::Result<()> {
		let from = args
			.from
			.as_deref()
			.map(|source| context.load(source, true))
			.transpose()?;
		let (pack, piece_name) = context.piece(&args.piece, from.as_ref())?;
		let piece = pack
			.piece(piece_name)
			.ok_or_else(|| anyhow!("no piece `{piece_name}`"))?;
		let mut values = inputs::parse(piece.manifest(), args.name.as_deref(), &args.inputs)?;
		if self.interactive {
			inputs::prompt_missing(piece.manifest(), &mut values)?;
		}
		let request = PieceRequest {
			pack,
			piece: piece_name,
			inputs: &values,
			variant: args.variant.as_deref(),
			target: args.target.as_ref(),
			granted: None,
			force: args.apply.force,
		};
		let plan = if adopt {
			context.engine.plan_adopt(&request)?
		} else {
			context.engine.plan(&request)?
		};
		self.finish(context, plan, &args.apply)
	}
}
