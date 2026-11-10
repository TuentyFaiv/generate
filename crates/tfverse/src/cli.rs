//! The command line: every command, argument and flag.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use tfverse_core::plan::OnConflict;
use tfverse_core::{RelPath, brand};

/// Generate projects and code from versioned, updatable packs, for humans and AI agents.
#[derive(Debug, Parser)]
#[command(name = brand::BINARY_NAME, version, about, long_about = None)]
pub(crate) struct Cli {
	/// Project root; defaults to the current folder.
	#[arg(long, global = true)]
	pub(crate) root: Option<PathBuf>,
	/// Print JSON instead of text.
	#[arg(long, global = true)]
	pub(crate) json: bool,
	/// Never ask questions; fail instead when something is missing.
	#[arg(long, global = true)]
	pub(crate) no_input: bool,
	#[command(subcommand)]
	pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
	/// Generate a piece, e.g. `tfverse add ui/atom Button`.
	Add(PieceArgs),
	/// Take over existing code as a piece, so later updates merge into it; existing files are kept.
	Adopt(PieceArgs),
	/// Update everything generated from a pack, merging your edits.
	Update(UpdateArgs),
	/// Show every generated file and whether it is clean, modified or missing.
	Status,
	/// Explain where a line of a generated file comes from, e.g. `tfverse why src/a.ts:12`.
	Why {
		/// `path:line`.
		location: String,
	},
	/// List the transactions that can be undone.
	Log,
	/// Undo the last transaction.
	Undo {
		/// Undo even if its files changed since; those edits are lost.
		#[arg(long)]
		force: bool,
	},
	/// Show the project's detected stack.
	Detect,
	/// Work with packs.
	#[command(subcommand)]
	Pack(PackCommand),
	/// Serve tfverse to AI agents over MCP (stdio).
	Mcp,
	/// Serve the engine over stdio JSON-RPC, for the GUI and other tools.
	#[command(hide = true)]
	Rpc,
}

#[derive(Debug, Subcommand)]
pub(crate) enum PackCommand {
	/// List the packs and their pieces.
	List,
	/// Show a pack's pieces and their inputs.
	Show {
		/// Pack name.
		pack: String,
	},
	/// Load and verify a pack from a source, e.g. `gh:acme/packs@v1.2.0`, `./packs/mine` or `registry:https://…/card.json`.
	Add {
		/// The source.
		source: String,
	},
	/// Turn project files into a new piece of a local pack.
	Capture(CaptureArgs),
}

#[derive(Debug, Args)]
pub(crate) struct PieceArgs {
	/// `pack/piece`, or a piece name found in one loaded pack.
	pub(crate) piece: String,
	/// Value of the piece's `name` input.
	pub(crate) name: Option<String>,
	/// Another input, `key=value`; repeat as needed.
	#[arg(short, long = "input", value_name = "KEY=VALUE")]
	pub(crate) inputs: Vec<String>,
	/// Load the pack from this source first.
	#[arg(long, value_name = "SOURCE")]
	pub(crate) from: Option<String>,
	/// Use this variant instead of the detected one.
	#[arg(long)]
	pub(crate) variant: Option<String>,
	/// Write into this folder instead of the piece's.
	#[arg(long)]
	pub(crate) target: Option<RelPath>,
	#[command(flatten)]
	pub(crate) apply: ApplyArgs,
}

#[derive(Debug, Args)]
pub(crate) struct UpdateArgs {
	/// Pack name.
	pub(crate) pack: String,
	/// Load the new version from this source.
	#[arg(long, value_name = "SOURCE")]
	pub(crate) from: Option<String>,
	/// What to do where your edits and the new version change the same lines.
	#[arg(long, value_enum, default_value_t = Conflicts::Refuse)]
	pub(crate) on_conflict: Conflicts,
	/// Do not run the pack's migrations.
	#[arg(long)]
	pub(crate) no_migrate: bool,
	#[command(flatten)]
	pub(crate) apply: ApplyArgs,
}

#[derive(Debug, Args)]
pub(crate) struct CaptureArgs {
	/// Project files to capture.
	#[arg(required = true)]
	pub(crate) files: Vec<RelPath>,
	/// Name of the new piece.
	#[arg(long)]
	pub(crate) piece: String,
	/// How the name is spelled in the files, e.g. `UserCard`; it becomes the `name` input.
	#[arg(long = "name", value_name = "SPELLING")]
	pub(crate) example: Option<String>,
	/// Folder of the pack to write into.
	#[arg(long, default_value = "packs/local")]
	pub(crate) pack_dir: RelPath,
	/// Pack name when that folder has no pack yet.
	#[arg(long, default_value = "local")]
	pub(crate) pack: String,
	#[command(flatten)]
	pub(crate) apply: ApplyArgs,
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools, reason = "independent command-line flags")]
pub(crate) struct ApplyArgs {
	/// Only show the plan.
	#[arg(long)]
	pub(crate) dry_run: bool,
	/// Apply without asking.
	#[arg(short, long)]
	pub(crate) yes: bool,
	/// Choose hunk by hunk what to keep, like `git add -p`.
	#[arg(long)]
	pub(crate) review: bool,
	/// Show diffs of changed files.
	#[arg(long)]
	pub(crate) diff: bool,
	/// Replace existing files that differ.
	#[arg(long)]
	pub(crate) force: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum Conflicts {
	/// Reject the plan.
	Refuse,
	/// Write Git-style conflict markers.
	Markers,
	/// Keep your lines.
	Ours,
	/// Take the new version's lines.
	Theirs,
}

impl From<Conflicts> for OnConflict {
	fn from(conflicts: Conflicts) -> Self {
		match conflicts {
			Conflicts::Refuse => Self::Refuse,
			Conflicts::Markers => Self::Markers,
			Conflicts::Ours => Self::Ours,
			Conflicts::Theirs => Self::Theirs,
		}
	}
}
