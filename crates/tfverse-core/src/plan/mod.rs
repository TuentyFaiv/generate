//! Plans: proposed changes, checked against the project and permissions before any write.

mod diff;
mod merge;
mod permissions;
mod review;

use std::fmt;

use serde::Serialize;

pub(crate) use diff::origin_line;
pub use merge::OnConflict;
pub(crate) use merge::{Labels, merge};
pub use review::{Hunk, Pick, SelectError, Selection};

use crate::model::{Dependency, Digest, Permissions, RelPath};
use crate::template;
use crate::workspace::{Instance, PackRecord, Workspace, WorkspaceError};

/// A plan still being built; it cannot be applied.
#[derive(Debug, Default)]
pub struct Draft {
	proposals: Vec<Proposal>,
	problems: Vec<Problem>,
}

/// Something the user should know about a plan that does not stop it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Note {
	/// The path involved.
	pub path: RelPath,
	/// What happened.
	pub message: String,
}

/// A plan checked against the project; the only kind that can be applied.
#[derive(Debug)]
pub struct Validated {
	changes: Vec<Change>,
}

/// Proposed changes in state `S`.
#[derive(Debug)]
pub struct Plan<S> {
	summary: String,
	record: Record,
	notes: Vec<Note>,
	state: S,
}

/// What a plan adds to the lockfile and object cache.
#[derive(Debug, Default)]
pub(crate) struct Record {
	/// The pack the pieces come from.
	pub pack: Option<PackRecord>,
	/// Generated pieces.
	pub instances: Vec<Instance>,
	/// Packages the pieces import.
	pub dependencies: Vec<Dependency>,
	/// Content to cache, such as patch regions.
	pub blobs: Vec<Vec<u8>>,
}

#[derive(Debug)]
enum Proposal {
	Create {
		path: RelPath,
		contents: Vec<u8>,
	},
	Modify {
		path: RelPath,
		before: Vec<u8>,
		after: Vec<u8>,
		conflicts: usize,
	},
	Delete {
		path: RelPath,
		before: Vec<u8>,
	},
}

/// One change in a validated plan.
#[derive(Clone, Debug, Serialize)]
pub struct Change {
	/// Project-relative path.
	pub path: RelPath,
	/// What happens to the file.
	pub action: Action,
	/// Why this path deserves attention, if it does.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub risk: Option<&'static str>,
	/// Unified diff for modified text files.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub diff: Option<String>,
	/// Regions where the user's edits and a pack's new version conflict.
	#[serde(skip_serializing_if = "is_zero")]
	pub conflicts: usize,
	#[serde(skip)]
	pub(crate) current: Option<Digest>,
	#[serde(skip)]
	pub(crate) before: Option<Vec<u8>>,
	#[serde(skip)]
	pub(crate) contents: Vec<u8>,
}

/// What a change does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Action {
	/// A new file.
	Create,
	/// Replaces an existing file, only with `force`.
	Overwrite,
	/// Edits an existing file.
	Modify,
	/// Removes a file.
	Delete,
	/// The file already has this content.
	Unchanged,
}

/// Limits a plan must respect.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rules<'a> {
	/// What the pack declares it may write.
	pub declared: &'a Permissions,
	/// What the user granted the pack.
	pub granted: &'a Permissions,
	/// Allow replacing existing files that differ.
	pub force: bool,
	/// What to do with conflicting merges.
	pub on_conflict: OnConflict,
}

/// Why a plan was rejected.
#[derive(Debug, thiserror::Error, Serialize)]
pub struct Rejected {
	/// Every problem found.
	pub problems: Vec<Problem>,
}

impl fmt::Display for Rejected {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "plan rejected")?;
		for problem in &self.problems {
			write!(f, "\n  {}: {}", problem.path, problem.message)?;
		}
		Ok(())
	}
}

/// One reason a plan cannot be applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Problem {
	/// The path involved.
	pub path: RelPath,
	/// Kind of problem.
	pub kind: ProblemKind,
	/// Human-readable explanation.
	pub message: String,
}

/// Kinds of plan problems.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ProblemKind {
	/// The file exists with other content, or changed while planning.
	Conflict,
	/// The pack does not declare permission to write here.
	NotDeclared,
	/// The user did not grant permission to write here.
	NotGranted,
	/// No pack may write here.
	Forbidden,
	/// The path goes through a symlink or a file.
	Unsafe,
	/// A tool the plan needs is not available.
	Unavailable,
}

/// A serializable summary of a validated plan.
#[derive(Clone, Debug, Serialize)]
pub struct PlanReport<'a> {
	/// What the plan does.
	pub summary: &'a str,
	/// Every change.
	pub changes: &'a [Change],
	/// Instances recorded in the lockfile.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub instances: Vec<&'a str>,
	/// Packages to add to the project; tfverse lists them but does not install them.
	#[serde(skip_serializing_if = "<[Dependency]>::is_empty")]
	pub dependencies: &'a [Dependency],
	/// Things to know that do not stop the plan.
	#[serde(skip_serializing_if = "<[Note]>::is_empty")]
	pub notes: &'a [Note],
}

impl Plan<Draft> {
	/// An empty plan.
	pub(crate) fn new(summary: impl Into<String>) -> Self {
		Self {
			summary: summary.into(),
			record: Record::default(),
			notes: Vec::new(),
			state: Draft::default(),
		}
	}

	/// Proposes a new file.
	pub(crate) fn create(&mut self, path: RelPath, contents: Vec<u8>) {
		self.state.proposals.push(Proposal::Create { path, contents });
	}

	/// Proposes editing a file whose current content is `before`.
	pub(crate) fn modify(&mut self, path: RelPath, before: Vec<u8>, after: Vec<u8>) {
		self.merged(path, before, after, 0);
	}

	/// Proposes the result of a merge with `conflicts` conflicting regions.
	pub(crate) fn merged(&mut self, path: RelPath, before: Vec<u8>, after: Vec<u8>, conflicts: usize) {
		self.state.proposals.push(Proposal::Modify {
			path,
			before,
			after,
			conflicts,
		});
	}

	/// Proposes removing a file whose current content is `before`.
	pub(crate) fn delete(&mut self, path: RelPath, before: Vec<u8>) {
		self.state.proposals.push(Proposal::Delete { path, before });
	}

	/// Rejects the plan for a reason found while planning.
	pub(crate) fn reject(&mut self, path: RelPath, kind: ProblemKind, message: impl Into<String>) {
		self.state.problems.push(Problem {
			path,
			kind,
			message: message.into(),
		});
	}

	/// Tells the user something that does not stop the plan.
	pub(crate) fn note(&mut self, path: RelPath, message: impl Into<String>) {
		self.notes.push(Note {
			path,
			message: message.into(),
		});
	}

	/// Records `instance` in the lockfile when applied.
	pub(crate) fn record(&mut self, instance: Instance) {
		self.record.instances.push(instance);
	}

	/// Records the pack the pieces come from.
	pub(crate) fn record_pack(&mut self, pack: PackRecord) {
		self.record.pack = Some(pack);
	}

	/// Lists `dependency` as needed, once.
	pub(crate) fn depend(&mut self, dependency: Dependency) {
		let same = |existing: &Dependency| existing.ecosystem == dependency.ecosystem && existing.name == dependency.name;
		if !self.record.dependencies.iter().any(same) {
			self.record.dependencies.push(dependency);
		}
	}

	/// Whether the plan already writes `path`.
	pub(crate) fn writes(&self, path: &RelPath) -> bool {
		self.state.proposals.iter().any(|proposal| proposal.path() == path)
	}

	/// Caches `blob`, such as a patch region, in the object store when applied.
	pub(crate) fn keep(&mut self, blob: Vec<u8>) {
		self.record.blobs.push(blob);
	}

	/// Checks every proposal against the project and `rules`.
	pub(crate) fn validate(self, workspace: &Workspace, rules: &Rules<'_>) -> Result<Plan<Validated>, Rejected> {
		let mut problems = self.state.problems;
		let mut changes = Vec::new();
		for proposal in self.state.proposals {
			match check(proposal, workspace, rules) {
				Ok(change) if change.conflicts > 0 && rules.on_conflict == OnConflict::Refuse => {
					problems.push(Problem {
						message: format!(
							"{} region(s) conflict with your edits; choose markers, ours or theirs to resolve them",
							change.conflicts
						),
						path: change.path,
						kind: ProblemKind::Conflict,
					});
				}
				Ok(change) => changes.push(change),
				Err(problem) => problems.push(problem),
			}
		}
		if !rules.granted.dependencies {
			problems.extend(self.record.dependencies.iter().map(|dependency| Problem {
				path: RelPath::constant(dependency.ecosystem.manifest()),
				kind: ProblemKind::NotGranted,
				message: format!(
					"you have not granted the pack permission to add the dependency `{}`",
					dependency.name
				),
			}));
		}

		if problems.is_empty() {
			Ok(Plan {
				summary: self.summary,
				record: self.record,
				notes: self.notes,
				state: Validated { changes },
			})
		} else {
			Err(Rejected { problems })
		}
	}
}

impl Plan<Validated> {
	/// What the plan does.
	#[must_use]
	pub fn summary(&self) -> &str {
		&self.summary
	}

	/// Every change, including unchanged files.
	#[must_use]
	pub fn changes(&self) -> &[Change] {
		&self.state.changes
	}

	/// The instances recorded on apply.
	#[must_use]
	pub fn instances(&self) -> &[Instance] {
		&self.record.instances
	}

	/// Packages the pieces import; tfverse does not install them.
	#[must_use]
	pub fn dependencies(&self) -> &[Dependency] {
		&self.record.dependencies
	}

	/// Things to know that do not stop the plan.
	#[must_use]
	pub fn notes(&self) -> &[Note] {
		&self.notes
	}

	/// A serializable summary.
	#[must_use]
	pub fn report(&self) -> PlanReport<'_> {
		PlanReport {
			summary: &self.summary,
			changes: &self.state.changes,
			instances: self
				.record
				.instances
				.iter()
				.map(|instance| instance.id.as_str())
				.collect(),
			dependencies: &self.record.dependencies,
			notes: &self.notes,
		}
	}

	pub(crate) fn into_parts(self) -> (String, Vec<Change>, Record) {
		(self.summary, self.state.changes, self.record)
	}
}

impl Proposal {
	fn path(&self) -> &RelPath {
		match self {
			Self::Create { path, .. } | Self::Modify { path, .. } | Self::Delete { path, .. } => path,
		}
	}
}

#[allow(clippy::trivially_copy_pass_by_ref, reason = "serde passes fields by reference")]
fn is_zero(count: &usize) -> bool {
	*count == 0
}

fn check(proposal: Proposal, workspace: &Workspace, rules: &Rules<'_>) -> Result<Change, Problem> {
	let path = proposal.path().clone();
	let problem = |kind, message: &str| Problem {
		path: path.clone(),
		kind,
		message: message.to_owned(),
	};

	if permissions::is_forbidden(&path) {
		return Err(problem(ProblemKind::Forbidden, "tfverse never lets a pack write here"));
	}
	if !rules.declared.allows_write(&path) {
		return Err(problem(
			ProblemKind::NotDeclared,
			"the pack does not declare permission to write here",
		));
	}
	if !rules.granted.allows_write(&path) {
		return Err(problem(
			ProblemKind::NotGranted,
			"you have not granted the pack permission to write here",
		));
	}
	let current = workspace.read(&path).map_err(|error| match error {
		WorkspaceError::Symlink(_) | WorkspaceError::NotADir(_) | WorkspaceError::NotAFile(_) => {
			problem(ProblemKind::Unsafe, &error.to_string())
		}
		WorkspaceError::Io { .. } => problem(ProblemKind::Conflict, &error.to_string()),
	})?;

	let mut conflicts = 0;
	let (action, contents) = match (proposal, &current) {
		(Proposal::Create { contents, .. }, None) => (Action::Create, contents),
		(Proposal::Create { contents, .. }, Some(existing)) if *existing == contents => (Action::Unchanged, contents),
		(Proposal::Create { contents, .. }, Some(_)) if rules.force => (Action::Overwrite, contents),
		(Proposal::Create { .. }, Some(_)) => {
			return Err(problem(
				ProblemKind::Conflict,
				"file already exists with other content; use --force to replace it",
			));
		}
		(
			Proposal::Modify {
				before,
				after,
				conflicts: found,
				..
			},
			Some(existing),
		) if *existing == before => {
			conflicts = found;
			(
				if after == before {
					Action::Unchanged
				} else {
					Action::Modify
				},
				after,
			)
		}
		(Proposal::Delete { before, .. }, Some(existing)) if *existing == before => (Action::Delete, Vec::new()),
		(Proposal::Delete { .. }, None) => (Action::Unchanged, Vec::new()),
		(Proposal::Modify { .. } | Proposal::Delete { .. }, _) => {
			return Err(problem(
				ProblemKind::Conflict,
				"file changed while planning; plan again",
			));
		}
	};

	let diff = current
		.as_deref()
		.filter(|_| action != Action::Unchanged)
		.and_then(|before| text_diff(before, &contents));
	Ok(Change {
		risk: permissions::risk(&path),
		current: current.as_deref().map(Digest::of),
		before: current,
		path,
		action,
		diff,
		conflicts,
		contents,
	})
}

fn text_diff(before: &[u8], after: &[u8]) -> Option<String> {
	if template::is_binary(before) || template::is_binary(after) {
		return None;
	}
	Some(diff::unified(
		&String::from_utf8_lossy(before),
		&String::from_utf8_lossy(after),
	))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::Glob;

	fn src_only() -> Permissions {
		Permissions {
			write: vec![Glob::new("src/**").unwrap()],
			..Permissions::default()
		}
	}

	fn rel(path: &str) -> RelPath {
		RelPath::new(path).unwrap()
	}

	fn validate(plan: Plan<Draft>, workspace: &Workspace, force: bool) -> Result<Plan<Validated>, Rejected> {
		let permissions = src_only();
		plan.validate(
			workspace,
			&Rules {
				declared: &permissions,
				granted: &permissions,
				force,
				on_conflict: OnConflict::Refuse,
			},
		)
	}

	fn setup() -> (tempfile::TempDir, Workspace) {
		let dir = tempfile::tempdir().unwrap();
		let workspace = Workspace::open(dir.path()).unwrap();
		(dir, workspace)
	}

	#[test]
	fn validate_should_mark_new_file_as_create() {
		let (_dir, workspace) = setup();
		let mut plan = Plan::new("test");
		plan.create(rel("src/a.ts"), b"x".to_vec());

		assert_eq!(
			validate(plan, &workspace, false).unwrap().changes()[0].action,
			Action::Create
		);
	}

	#[test]
	fn validate_should_reject_existing_different_file() {
		let (_dir, workspace) = setup();
		workspace.write(&rel("src/a.ts"), b"mine").unwrap();
		let mut plan = Plan::new("test");
		plan.create(rel("src/a.ts"), b"x".to_vec());

		assert_eq!(
			validate(plan, &workspace, false).unwrap_err().problems[0].kind,
			ProblemKind::Conflict
		);
	}

	#[test]
	fn validate_should_overwrite_with_force_and_show_diff() {
		let (_dir, workspace) = setup();
		workspace.write(&rel("src/a.ts"), b"mine\n").unwrap();
		let mut plan = Plan::new("test");
		plan.create(rel("src/a.ts"), b"theirs\n".to_vec());

		let change = &validate(plan, &workspace, true).unwrap().changes()[0].clone();

		assert_eq!(
			(change.action, change.diff.as_deref()),
			(Action::Overwrite, Some("@@ -1,1 +1,1 @@\n-mine\n+theirs\n"))
		);
	}

	#[test]
	fn validate_should_mark_identical_file_as_unchanged() {
		let (_dir, workspace) = setup();
		workspace.write(&rel("src/a.ts"), b"x").unwrap();
		let mut plan = Plan::new("test");
		plan.create(rel("src/a.ts"), b"x".to_vec());

		assert_eq!(
			validate(plan, &workspace, false).unwrap().changes()[0].action,
			Action::Unchanged
		);
	}

	#[test]
	fn validate_should_reject_undeclared_path() {
		let (_dir, workspace) = setup();
		let mut plan = Plan::new("test");
		plan.create(rel("lib/a.ts"), b"x".to_vec());

		assert_eq!(
			validate(plan, &workspace, true).unwrap_err().problems[0].kind,
			ProblemKind::NotDeclared
		);
	}

	#[test]
	fn validate_should_reject_ungranted_path() {
		let (_dir, workspace) = setup();
		let declared = src_only();
		let granted = Permissions {
			write: vec![Glob::new("src/ui/**").unwrap()],
			..Permissions::default()
		};
		let mut plan = Plan::new("test");
		plan.create(rel("src/a.ts"), b"x".to_vec());

		let rejected = plan
			.validate(
				&workspace,
				&Rules {
					declared: &declared,
					granted: &granted,
					force: false,
					on_conflict: OnConflict::Refuse,
				},
			)
			.unwrap_err();

		assert_eq!(rejected.problems[0].kind, ProblemKind::NotGranted);
	}

	#[test]
	fn validate_should_forbid_git_even_when_granted() {
		let (_dir, workspace) = setup();
		let everything = Permissions {
			write: vec![Glob::new("**").unwrap()],
			..Permissions::default()
		};
		let mut plan = Plan::new("test");
		plan.create(rel(".git/hooks/pre-commit"), b"x".to_vec());

		let rejected = plan
			.validate(
				&workspace,
				&Rules {
					declared: &everything,
					granted: &everything,
					force: true,
					on_conflict: OnConflict::Refuse,
				},
			)
			.unwrap_err();

		assert_eq!(rejected.problems[0].kind, ProblemKind::Forbidden);
	}

	#[test]
	fn validate_should_reject_modify_when_file_changed() {
		let (_dir, workspace) = setup();
		workspace.write(&rel("src/a.ts"), b"changed").unwrap();
		let mut plan = Plan::new("test");
		plan.modify(rel("src/a.ts"), b"original".to_vec(), b"patched".to_vec());

		assert_eq!(
			validate(plan, &workspace, false).unwrap_err().problems[0].kind,
			ProblemKind::Conflict
		);
	}
}
