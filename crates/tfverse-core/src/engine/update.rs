//! Updating generated pieces to another version of their pack, merging each file with the user's edits.

use std::collections::BTreeMap;

use semver::Version;

use super::planner::{Added, check_pinned, collect_required, finish};
use super::{Engine, EngineError, PackProvider};
use crate::migrate::MigrateError;
use crate::model::{Digest, Migration, Name, Permissions, RelPath, Values};
use crate::pack::{Pack, Piece, RenderRequest, Rendered, render, resolve_inputs};
use crate::patch::Patch;
use crate::plan::{Draft, Labels, OnConflict, Plan, ProblemKind, Validated, merge};
use crate::template;
use crate::workspace::{FileRecord, Instance, Lockfile, PatchRecord};

/// The most project files one update's migrations look at.
const MAX_MIGRATED_FILES: usize = 20_000;

/// A request to move every piece generated from a pack to another version of it.
#[derive(Clone, Copy, Debug)]
pub struct UpdateRequest<'a> {
	/// The version to update to.
	pub pack: &'a Pack,
	/// Permissions the user granted; defaults to what this version declares.
	pub granted: Option<&'a Permissions>,
	/// What to do where the user's edits and this version change the same lines.
	pub on_conflict: OnConflict,
	/// Allow replacing existing files this version adds.
	pub force: bool,
	/// Run the migrations this update crosses.
	pub migrate: bool,
}

/// Old records and new patches of one file, each with its instance index.
type FilePatches<'a> = (Vec<(usize, &'a PatchRecord)>, Vec<(usize, &'a Patch)>);

/// One instance's patches before and after the update.
struct PatchUpdate {
	index: usize,
	old: Vec<PatchRecord>,
	new: Vec<Patch>,
}

impl Engine {
	/// Plans updating every piece from `request.pack`'s pack to that version; nothing is written.
	///
	/// Files the user did not edit take the new version; edited files are merged line by line, and
	/// `packs` provides the old version when the original of a file is not cached.
	pub fn plan_update<P: PackProvider + ?Sized>(
		&self,
		request: &UpdateRequest<'_>,
		packs: &P,
	) -> Result<Plan<Validated>, EngineError> {
		let pack = request.pack;
		let manifest = pack.manifest();
		let lock = self.lockfile()?;
		check_pinned(pack, &lock)?;
		let record = lock
			.pack(&manifest.name)
			.ok_or_else(|| EngineError::NotInstalled(manifest.name.clone()))?;
		let mut plan = Plan::new(format!(
			"update {} {} → {}",
			manifest.name, record.version, manifest.version
		));
		let theirs_label = format!("{} {}", manifest.name, manifest.version);
		let labels = Labels {
			ours: "yours",
			theirs: &theirs_label,
		};

		let mut updated = Vec::new();
		let mut patch_updates = Vec::new();
		let mut pieces = Vec::new();
		for old in lock.instances.iter().filter(|instance| instance.pack == manifest.name) {
			let Some(piece) = pack.piece(old.piece.as_str()) else {
				plan.note(
					Lockfile::path(),
					format!(
						"{} {} no longer has the piece `{}`; its files stay as they are",
						manifest.name, manifest.version, old.piece
					),
				);
				continue;
			};
			let previous = packs
				.pack(&old.pack, &old.pack_version)
				.and_then(|previous| previous.piece(old.piece.as_str()))
				.map(|previous| render_instance(previous, old, &old.inputs))
				.transpose()?;
			let values = carried_inputs(piece, &old.inputs)?;
			let next = render_instance(piece, old, &values)?;
			for dependency in &next.dependencies {
				if previous
					.as_ref()
					.is_none_or(|previous| !previous.dependencies.contains(dependency))
				{
					plan.depend(dependency.clone());
				}
			}
			let files = self.update_files(&mut plan, request, labels, old, previous.as_ref(), &next)?;
			patch_updates.push(PatchUpdate {
				index: updated.len(),
				old: old.patches.clone(),
				new: next.patches,
			});
			updated.push(Instance {
				pack_version: manifest.version.clone(),
				variant: next.variant,
				target: next.target,
				inputs: values,
				files,
				patches: Vec::new(),
				..old.clone()
			});
			pieces.push(piece);
		}

		for (index, record) in self.update_patches(&mut plan, &patch_updates)? {
			updated[index].patches.push(record);
		}
		for instance in updated {
			plan.record(instance);
		}
		let from = record.version.clone();
		if request.migrate {
			self.migrate(&mut plan, pack, &from)?;
		}
		self.add_required(&mut plan, pack, &pieces, &lock)?;
		finish(
			plan,
			&self.workspace,
			pack,
			request.granted,
			request.force,
			request.on_conflict,
		)
	}

	/// Plans each file of one instance and returns the records the lockfile keeps for it.
	fn update_files(
		&self,
		plan: &mut Plan<Draft>,
		request: &UpdateRequest<'_>,
		labels: Labels<'_>,
		old: &Instance,
		previous: Option<&Rendered>,
		next: &Rendered,
	) -> Result<Vec<FileRecord>, EngineError> {
		let store = self.store();
		let mut theirs: BTreeMap<&RelPath, &[u8]> = next
			.files
			.iter()
			.map(|file| (&file.path, file.contents.as_slice()))
			.collect();
		let mut records = Vec::new();
		for record in &old.files {
			let path = &record.path;
			let current = self.workspace.read(path)?;
			let Some(theirs) = theirs.remove(path) else {
				match current {
					Some(current) if Digest::of(&current) == record.base => plan.delete(path.clone(), current),
					Some(_) => plan.note(
						path.clone(),
						"the new version no longer has this file; you edited it, so it stays and is no longer tracked",
					),
					None => {}
				}
				continue;
			};
			records.push(FileRecord {
				path: path.clone(),
				base: Digest::of(theirs),
			});
			plan.keep(theirs.to_vec());
			let Some(current) = current else {
				if Digest::of(theirs) != record.base {
					plan.note(path.clone(), "you deleted this file, so it stays deleted");
				}
				continue;
			};
			let current_digest = Digest::of(&current);
			if Digest::of(theirs) == record.base || current.as_slice() == theirs {
				continue;
			}
			if current_digest == record.base {
				plan.modify(path.clone(), current, theirs.to_vec());
				continue;
			}
			let base = match store.get(&record.base)? {
				Some(base) => Some(base),
				None => previous
					.and_then(|previous| previous.files.iter().find(|file| file.path == *path))
					.filter(|file| Digest::of(&file.contents) == record.base)
					.map(|file| file.contents.clone()),
			};
			merge_file(plan, request.on_conflict, labels, path, base, current, theirs);
		}
		for (path, contents) in theirs {
			if plan.writes(path) {
				return Err(EngineError::Overlap(path.clone()));
			}
			records.push(FileRecord {
				path: path.clone(),
				base: Digest::of(contents),
			});
			plan.create(path.clone(), contents.to_vec());
		}
		Ok(records)
	}

	/// Replaces each instance's old patch regions with its new ones, file by file.
	fn update_patches(
		&self,
		plan: &mut Plan<Draft>,
		updates: &[PatchUpdate],
	) -> Result<Vec<(usize, PatchRecord)>, EngineError> {
		let mut by_file: BTreeMap<&RelPath, FilePatches<'_>> = BTreeMap::new();
		for update in updates {
			for record in &update.old {
				by_file.entry(&record.file).or_default().0.push((update.index, record));
			}
			for patch in &update.new {
				by_file.entry(patch.file()).or_default().1.push((update.index, patch));
			}
		}
		let store = self.store();
		let mut records = Vec::new();
		for (file, (old, new)) in by_file {
			let Some(before) = self.workspace.read(file)? else {
				plan.note(file.clone(), "this file is gone, so its patches were not updated");
				continue;
			};
			let mut text = String::from_utf8_lossy(&before).into_owned();
			let mut intact = true;
			for (_, record) in &old {
				match store.get(&record.inserted)? {
					Some(region) if text.contains(&*String::from_utf8_lossy(&region)) => {
						text = text.replacen(&*String::from_utf8_lossy(&region), "", 1);
					}
					_ => intact = false,
				}
			}
			if !intact {
				plan.note(
					file.clone(),
					"you edited a region tfverse inserted here, so this file's patches were not updated",
				);
				records.extend(old.into_iter().map(|(index, record)| (index, record.clone())));
				continue;
			}
			for (index, patch) in new {
				records.push((index, self.patch(plan, &mut text, patch)?));
			}
			plan.modify(file.clone(), before, text.into_bytes());
		}
		Ok(records)
	}

	/// Runs every migration between `from` and the new version on the project files it covers.
	///
	/// Files the update itself changes are left alone and noted, so the two never fight over a file.
	fn migrate(&self, plan: &mut Plan<Draft>, pack: &Pack, from: &Version) -> Result<(), EngineError> {
		let manifest = pack.manifest();
		let mut due: Vec<&Migration> = manifest
			.migrations
			.iter()
			.filter(|migration| *from < migration.to && migration.to <= manifest.version)
			.collect();
		if due.is_empty() {
			return Ok(());
		}
		due.sort_by(|a, b| a.to.cmp(&b.to));
		let mut changed = vec![0_usize; due.len()];
		for path in self.workspace.walk(MAX_MIGRATED_FILES)? {
			let applicable: Vec<usize> = (0..due.len())
				.filter(|index| due[*index].files.iter().any(|glob| glob.matches(&path)))
				.collect();
			if applicable.is_empty() {
				continue;
			}
			if plan.writes(&path) {
				plan.note(path, "the update changes this file, so its migrations did not run here");
				continue;
			}
			let Some(before) = self.workspace.read(&path)? else {
				continue;
			};
			if template::is_binary(&before) {
				continue;
			}
			let mut text = String::from_utf8_lossy(&before).into_owned();
			for index in applicable {
				let migration = due[index];
				let rewritten = match (&migration.rename, &migration.ast_grep) {
					(Some(rename), _) => Ok(crate::migrate::rename(&text, &rename.from, &rename.to)),
					(None, Some(rule)) => match pack.migration_rule(rule) {
						Some(rule) => self.ast_grep.rewrite(rule, &text),
						None => Ok(None),
					},
					(None, None) => Ok(None),
				};
				match rewritten {
					Ok(Some(next)) => {
						text = next;
						changed[index] += 1;
					}
					Ok(None) => {}
					Err(MigrateError::Missing) => {
						plan.reject(
							Lockfile::path(),
							ProblemKind::Unavailable,
							MigrateError::Missing.to_string(),
						);
						return Ok(());
					}
					Err(error) => {
						plan.reject(path.clone(), ProblemKind::Unavailable, error.to_string());
					}
				}
			}
			if text.as_bytes() != before.as_slice() {
				plan.modify(path, before, text.into_bytes());
			}
		}
		for (migration, count) in due.iter().zip(changed) {
			plan.note(
				Lockfile::path(),
				format!(
					"migration to {}: {} ({count} file(s) changed)",
					migration.to, migration.description
				),
			);
		}
		Ok(())
	}

	/// Adds the pieces the new version requires that the project does not have yet.
	fn add_required(
		&self,
		plan: &mut Plan<Draft>,
		pack: &Pack,
		pieces: &[&Piece],
		lock: &Lockfile,
	) -> Result<(), EngineError> {
		let mut required = Vec::new();
		for piece in pieces {
			collect_required(pack, piece, lock, &mut required);
		}
		if required.is_empty() {
			return Ok(());
		}
		let stack = self.project()?.stack;
		let mut added = Added::default();
		for piece in required {
			let values = self.values(piece.manifest().name.as_str(), piece, None)?;
			self.add_piece(plan, &mut added, pack, piece, &values, None, None, &stack)?;
		}
		for (index, record) in self.add_patches(plan, &added.patches)? {
			added.instances[index].patches.push(record);
		}
		for instance in added.instances {
			plan.record(instance);
		}
		Ok(())
	}
}

/// Plans a file both the user and the new version changed.
#[allow(clippy::too_many_arguments)]
fn merge_file(
	plan: &mut Plan<Draft>,
	on_conflict: OnConflict,
	labels: Labels<'_>,
	path: &RelPath,
	base: Option<Vec<u8>>,
	current: Vec<u8>,
	theirs: &[u8],
) {
	let binary = template::is_binary(&current) || template::is_binary(theirs);
	match (base, binary, on_conflict) {
		(Some(base), false, _) if !template::is_binary(&base) => {
			let merged = merge(
				&String::from_utf8_lossy(&base),
				&String::from_utf8_lossy(&current),
				&String::from_utf8_lossy(theirs),
				on_conflict,
				labels,
			);
			plan.merged(path.clone(), current, merged.text.into_bytes(), merged.conflicts);
		}
		(_, _, OnConflict::Ours) => plan.note(
			path.clone(),
			"kept your version; the new version's changes were skipped",
		),
		(_, _, OnConflict::Theirs) => plan.merged(path.clone(), current, theirs.to_vec(), 1),
		(None, false, OnConflict::Markers) => {
			let merged = merge(
				"",
				&String::from_utf8_lossy(&current),
				&String::from_utf8_lossy(theirs),
				on_conflict,
				labels,
			);
			plan.merged(path.clone(), current, merged.text.into_bytes(), merged.conflicts.max(1));
		}
		(_, true, _) | (_, false, OnConflict::Refuse | OnConflict::Markers) => plan.reject(
			path.clone(),
			ProblemKind::Conflict,
			if binary {
				"you changed this binary file and so did the new version; choose ours or theirs"
			} else {
				"you edited this file and its original is not available to merge with; choose markers, ours or theirs"
			},
		),
	}
}

/// Renders `piece` with an instance's recorded stack, variant and target.
fn render_instance(piece: &Piece, instance: &Instance, values: &Values) -> Result<Rendered, EngineError> {
	Ok(render(&RenderRequest {
		piece,
		values,
		stack: &instance.stack,
		variant: instance.variant.as_ref().map(Name::as_str),
		target: instance.target.as_ref(),
	})?)
}

/// The recorded inputs `piece` still declares, with defaults for the ones it added.
fn carried_inputs(piece: &Piece, recorded: &Values) -> Result<Values, EngineError> {
	let declared = &piece.manifest().inputs;
	let kept: Values = recorded
		.iter()
		.filter(|(name, _)| declared.contains_key(*name))
		.map(|(name, value)| (name.clone(), value.clone()))
		.collect();
	Ok(resolve_inputs(declared, &kept)?)
}
