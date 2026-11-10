//! Planning a piece and the pieces it requires: inputs → render → patches → lock records → validation.

use std::collections::BTreeMap;

use super::{Engine, EngineError, PieceRequest};
use crate::model::{Digest, Name, Permissions, RelPath, Stack, Values};
use crate::pack::{Pack, Piece, RenderRequest, Rendered, RenderedFile, render, resolve_inputs};
use crate::patch::{Patch, Patched};
use crate::plan::{Draft, OnConflict, Plan, Rules, Validated};
use crate::workspace::{FileRecord, Instance, Lockfile, PackRecord, PatchRecord, instance_id};

impl Engine {
	/// Plans generating a piece, and the pieces it requires that the project lacks; nothing is written.
	pub fn plan(&self, request: &PieceRequest<'_>) -> Result<Plan<Validated>, EngineError> {
		self.plan_pieces(request, false)
	}

	/// Plans taking over code that already exists: files the piece would write that the project has
	/// are recorded as generated, with what the pack would have written as their original, so later
	/// updates merge into them; missing files are created. Existing files are never changed.
	pub fn plan_adopt(&self, request: &PieceRequest<'_>) -> Result<Plan<Validated>, EngineError> {
		self.plan_pieces(request, true)
	}

	fn plan_pieces(&self, request: &PieceRequest<'_>, adopt: bool) -> Result<Plan<Validated>, EngineError> {
		let pack = request.pack;
		let manifest = pack.manifest();
		let lock = self.lockfile()?;
		check_pinned(pack, &lock)?;

		let main = pack.piece(request.piece).ok_or_else(|| EngineError::UnknownPiece {
			pack: manifest.name.to_string(),
			piece: request.piece.to_owned(),
		})?;
		let mut pieces = Vec::new();
		collect_required(pack, main, &lock, &mut pieces);
		pieces.push(main);

		let stack = self.project()?.stack;
		let verb = if adopt { "adopt" } else { "add" };
		let mut plan = Plan::new(summary(verb, manifest.name.as_str(), &pieces));
		let mut added = Added {
			adopt,
			..Added::default()
		};
		for piece in &pieces {
			let is_main = std::ptr::eq(*piece, main);
			let name = piece.manifest().name.as_str();
			let values = self.values(name, piece, if is_main { Some(request.inputs) } else { None })?;
			let variant = request
				.variant
				.filter(|variant| is_main || piece.manifest().variants.iter().any(|v| v.name.as_str() == *variant));
			let target = if is_main { request.target } else { None };
			self.add_piece(&mut plan, &mut added, pack, piece, &values, variant, target, &stack)?;
		}
		for (index, record) in self.add_patches(&mut plan, &added.patches)? {
			added.instances[index].patches.push(record);
		}
		for instance in added.instances {
			plan.record(instance);
		}
		finish(
			plan,
			&self.workspace,
			pack,
			request.granted,
			request.force,
			OnConflict::Refuse,
		)
	}

	/// Renders a new `piece` into `plan`; its patches wait in `added` until every piece is rendered.
	#[allow(clippy::too_many_arguments)]
	pub(super) fn add_piece(
		&self,
		plan: &mut Plan<Draft>,
		added: &mut Added,
		pack: &Pack,
		piece: &Piece,
		values: &Values,
		variant: Option<&str>,
		target: Option<&RelPath>,
		stack: &Stack,
	) -> Result<(), EngineError> {
		let rendered = render(&RenderRequest {
			piece,
			values,
			stack,
			variant,
			target: target.or_else(|| self.config.paths.get(piece.manifest().name.as_str())),
		})?;
		let Rendered {
			variant,
			target,
			files,
			patches,
			dependencies,
		} = rendered;
		for dependency in dependencies {
			plan.depend(dependency);
		}
		let mut instance = instance(pack, piece, variant, target, stack.clone(), values.clone());
		instance.files = if added.adopt {
			self.adopt_files(plan, files)?
		} else {
			add_files(plan, files)?
		};
		let index = added.instances.len();
		added.patches.extend(patches.into_iter().map(|patch| (index, patch)));
		added.instances.push(instance);
		Ok(())
	}

	pub(super) fn values(&self, name: &str, piece: &Piece, given: Option<&Values>) -> Result<Values, EngineError> {
		let mut values = self.config.defaults.get(name).cloned().unwrap_or_default();
		values.extend(
			given
				.into_iter()
				.flatten()
				.map(|(name, value)| (name.clone(), value.clone())),
		);
		Ok(resolve_inputs(&piece.manifest().inputs, &values)?)
	}

	/// Applies every patch, grouped by file, and returns each patch's record with its instance index.
	pub(super) fn add_patches(
		&self,
		plan: &mut Plan<Draft>,
		patches: &[(usize, Patch)],
	) -> Result<Vec<(usize, PatchRecord)>, EngineError> {
		let mut by_file: BTreeMap<&RelPath, Vec<&(usize, Patch)>> = BTreeMap::new();
		for entry in patches {
			by_file.entry(entry.1.file()).or_default().push(entry);
		}
		let mut records = Vec::new();
		for (file, file_patches) in by_file {
			let before = self
				.workspace
				.read(file)?
				.ok_or_else(|| EngineError::PatchTarget(file.clone()))?;
			let mut text = String::from_utf8_lossy(&before).into_owned();
			for (index, patch) in file_patches {
				records.push((*index, self.patch(plan, &mut text, patch)?));
			}
			plan.modify(file.clone(), before, text.into_bytes());
		}
		Ok(records)
	}

	/// Records the rendered files the project already has, keeping their content; creates the rest.
	fn adopt_files(&self, plan: &mut Plan<Draft>, files: Vec<RenderedFile>) -> Result<Vec<FileRecord>, EngineError> {
		let mut records = Vec::with_capacity(files.len());
		let mut missing = Vec::new();
		for file in files {
			match self.workspace.read(&file.path)? {
				Some(current) => {
					if current != file.contents {
						plan.note(
							file.path.clone(),
							"adopted with your differences; updates will merge into them",
						);
					}
					records.push(FileRecord {
						path: file.path,
						base: Digest::of(&file.contents),
					});
					plan.keep(file.contents);
				}
				None => missing.push(file),
			}
		}
		records.extend(add_files(plan, missing)?);
		Ok(records)
	}

	/// Applies `patch` to `text` and returns the record of the region it owns.
	pub(super) fn patch(
		&self,
		plan: &mut Plan<Draft>,
		text: &mut String,
		patch: &Patch,
	) -> Result<PatchRecord, EngineError> {
		let inserted = match self.patchers.apply(text, patch) {
			Ok(Patched::Changed { text: next, inserted }) => {
				*text = next;
				inserted
			}
			Ok(Patched::AlreadyApplied { inserted }) => inserted,
			Err(source) => {
				return Err(EngineError::Patch {
					file: patch.file().clone(),
					source,
				});
			}
		};
		let record = PatchRecord {
			file: patch.file().clone(),
			inserted: Digest::of(inserted.as_bytes()),
		};
		plan.keep(inserted.into_bytes());
		Ok(record)
	}
}

/// New instances and their patches, applied once every piece is rendered.
#[derive(Debug, Default)]
pub(super) struct Added {
	/// Take over existing files instead of creating them.
	pub adopt: bool,
	/// New instances.
	pub instances: Vec<Instance>,
	/// Each patch with the index of its instance.
	pub patches: Vec<(usize, Patch)>,
}

/// Refuses a pack whose files differ from those the lockfile pinned for the same version, or that is
/// not signed by the key that signed the pinned version.
pub(super) fn check_pinned(pack: &Pack, lock: &Lockfile) -> Result<(), EngineError> {
	let manifest = pack.manifest();
	match lock.pack(&manifest.name) {
		Some(record) if record.version == manifest.version && record.digest != pack.digest() => {
			Err(EngineError::PackChanged {
				name: manifest.name.clone(),
				version: manifest.version.clone(),
			})
		}
		Some(PackRecord {
			signer: Some(expected), ..
		}) if pack.signer() != Some(expected.as_str()) => Err(EngineError::SignerChanged {
			name: manifest.name.clone(),
			expected: expected.clone(),
			actual: pack.signer().map(str::to_owned),
		}),
		_ => Ok(()),
	}
}

/// Records `pack` and validates `plan` against the permissions it declares and the user granted.
pub(super) fn finish(
	mut plan: Plan<Draft>,
	workspace: &crate::workspace::Workspace,
	pack: &Pack,
	granted: Option<&Permissions>,
	force: bool,
	on_conflict: OnConflict,
) -> Result<Plan<Validated>, EngineError> {
	let manifest = pack.manifest();
	plan.record_pack(PackRecord {
		name: manifest.name.clone(),
		version: manifest.version.clone(),
		source: pack.source().map(str::to_owned),
		digest: pack.digest(),
		signer: pack.signer().map(str::to_owned),
	});
	let declared = &manifest.permissions;
	let rules = Rules {
		declared,
		granted: granted.unwrap_or(declared),
		force,
		on_conflict,
	};
	Ok(plan.validate(workspace, &rules)?)
}

/// Adds the pieces `piece` requires, deepest first, skipping those the project or the list already has.
pub(super) fn collect_required<'a>(pack: &'a Pack, piece: &'a Piece, lock: &Lockfile, pieces: &mut Vec<&'a Piece>) {
	for name in &piece.manifest().requires {
		let Some(required) = pack.piece(name.as_str()) else {
			continue;
		};
		let listed = pieces.iter().any(|listed| listed.manifest().name == *name);
		if !listed && !lock.has_piece(&pack.manifest().name, name) {
			collect_required(pack, required, lock, pieces);
			if !pieces.iter().any(|listed| listed.manifest().name == *name) {
				pieces.push(required);
			}
		}
	}
}

fn summary(verb: &str, pack: &str, pieces: &[&Piece]) -> String {
	let names: Vec<&str> = pieces.iter().map(|piece| piece.manifest().name.as_str()).collect();
	match names.split_last() {
		Some((main, [])) => format!("{verb} {pack}/{main}"),
		Some((main, required)) => format!("{verb} {pack}/{main} (with {})", required.join(", ")),
		None => format!("{verb} {pack}"),
	}
}

fn add_files(plan: &mut Plan<Draft>, files: Vec<RenderedFile>) -> Result<Vec<FileRecord>, EngineError> {
	let mut records = Vec::with_capacity(files.len());
	for file in files {
		if plan.writes(&file.path) {
			return Err(EngineError::Overlap(file.path));
		}
		records.push(FileRecord {
			path: file.path.clone(),
			base: Digest::of(&file.contents),
		});
		plan.create(file.path, file.contents);
	}
	Ok(records)
}

fn instance(
	pack: &Pack,
	piece: &Piece,
	variant: Option<Name>,
	target: Option<RelPath>,
	stack: Stack,
	values: Values,
) -> Instance {
	let manifest = pack.manifest();
	Instance {
		id: instance_id(&manifest.name, &piece.manifest().name, &values),
		pack: manifest.name.clone(),
		pack_version: manifest.version.clone(),
		piece: piece.manifest().name.clone(),
		variant,
		target,
		stack,
		inputs: values,
		files: Vec::new(),
		patches: Vec::new(),
	}
}
