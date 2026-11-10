//! `pack list`, `pack show`, `pack add` and `pack capture`.

use anyhow::{Context as _, anyhow};
use serde_json::json;
use tfverse_core::{CaptureRequest, Pack};

use super::Session;
use crate::cli::PackCommand;
use crate::context::Context;
use crate::ui::Output;

impl Session {
	pub(super) fn pack(self, context: &mut Context, command: &PackCommand) -> anyhow::Result<()> {
		match command {
			PackCommand::List => {
				let packs: Vec<_> = context.library.iter().map(summary).collect();
				if self.json {
					Output::json(&packs);
				} else {
					for pack in context.library.iter() {
						let manifest = pack.manifest();
						println!(
							"{} {}  {}",
							manifest.name,
							manifest.version,
							pack.source().unwrap_or("local")
						);
						let pieces: Vec<&str> = pack.pieces().map(|piece| piece.manifest().name.as_str()).collect();
						println!("  {}", pieces.join(", "));
					}
				}
				Ok(())
			}
			PackCommand::Show { pack } => {
				let pack = context
					.library
					.get(pack)
					.ok_or_else(|| anyhow!("no pack `{pack}` is loaded"))?;
				if self.json {
					let pieces: Vec<_> = pack.pieces().map(tfverse_core::pack::Piece::manifest).collect();
					Output::json(&json!({ "pack": summary(pack), "pieces": pieces }));
					return Ok(());
				}
				let manifest = pack.manifest();
				println!("{} {}", manifest.name, manifest.version);
				if let Some(description) = &manifest.description {
					println!("{description}");
				}
				for piece in pack.pieces() {
					let piece = piece.manifest();
					println!(
						"\n{}/{}  {}",
						manifest.name,
						piece.name,
						piece.description.as_deref().unwrap_or("")
					);
					for (name, input) in &piece.inputs {
						println!("  -i {name}=…  {}", input.description().unwrap_or(""));
					}
					let variants: Vec<&str> = piece.variants.iter().map(|variant| variant.name.as_str()).collect();
					if !variants.is_empty() {
						println!("  variants: {}", variants.join(", "));
					}
				}
				Ok(())
			}
			PackCommand::Add { source } => {
				let pack = context.load(source, true)?;
				if self.json {
					Output::json(&summary(&pack));
				} else {
					println!(
						"Loaded {} {} from {} ({})",
						pack.manifest().name,
						pack.manifest().version,
						pack.source().unwrap_or(source),
						pack.signer().map_or_else(
							|| "not signed by a trusted key".to_owned(),
							|key| format!("signed by {key}")
						)
					);
				}
				Ok(())
			}
			PackCommand::Capture(args) => {
				let plan = context
					.engine
					.plan_capture(&CaptureRequest {
						pack_dir: &args.pack_dir,
						pack: &args.pack,
						piece: &args.piece,
						files: &args.files,
						example: args.example.as_deref(),
						force: args.apply.force,
					})
					.context("cannot capture these files")?;
				self.finish(context, plan, &args.apply)
			}
		}
	}
}

fn summary(pack: &Pack) -> serde_json::Value {
	let manifest = pack.manifest();
	json!({
		"name": manifest.name,
		"version": manifest.version.to_string(),
		"source": pack.source(),
		"digest": pack.digest(),
		"signer": pack.signer(),
		"description": manifest.description,
		"pieces": pack.pieces().map(|piece| piece.manifest().name.clone()).collect::<Vec<_>>(),
	})
}
