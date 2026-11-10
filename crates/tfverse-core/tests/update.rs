//! Updating generated pieces to a new pack version.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use tfverse_core::engine::{FileState, UpdateRequest};
use tfverse_core::migrate::{AstGrep, MigrateError};
use tfverse_core::plan::{Action, OnConflict, Pick, ProblemKind, SelectError, Selection};
use tfverse_core::{Engine, EngineError, Pack, PieceRequest, RelPath, UserConfig, Value, Values};

type TestResult = Result<(), Box<dyn Error>>;

const ORIGINAL: &str = "one\ntwo\nthree\nfour\nfive\n";
const ROUTES: &str = "export const routes = [\n\t// @tfverse:routes\n];\n";

/// Version `version` of a pack whose `page` piece renders `files` (relative to `src/pages`) and `extra` pack files.
fn pack(version: &str, files: &[(&str, &str)], extra: &[(&str, &str)]) -> Result<Pack, Box<dyn Error>> {
	let mut all = BTreeMap::from([
		(
			"pack.toml".to_owned(),
			format!("schema_version = 1\nname = \"site\"\nversion = \"{version}\"\n[permissions]\nwrite = [\"src/**\"]\n")
				.into_bytes(),
		),
		(
			"pieces/page/piece.toml".to_owned(),
			b"schema_version = 1\nname = \"page\"\ntarget = \"src/pages\"\n[inputs.name]\ntype = \"string\"\nexample = \"MyPage\"\n"
				.to_vec(),
		),
	]);
	for (path, text) in files {
		all.insert(format!("pieces/page/files/{path}"), text.as_bytes().to_vec());
	}
	for (path, text) in extra {
		all.insert((*path).to_owned(), text.as_bytes().to_vec());
	}
	Ok(Pack::from_files(all)?)
}

fn page(version: &str, text: &str) -> Result<Pack, Box<dyn Error>> {
	pack(version, &[("my-page.ts", text)], &[])
}

fn open(root: &Path) -> Result<Engine, EngineError> {
	Engine::builder().user_config(UserConfig::default()).open(root)
}

fn name() -> Values {
	Values::from([("name".to_owned(), Value::String("Home".to_owned()))])
}

/// A project with `pack`'s `page` piece applied.
fn generated(pack: &Pack) -> Result<(tempfile::TempDir, Engine), Box<dyn Error>> {
	let dir = tempfile::tempdir()?;
	fs::create_dir(dir.path().join("src"))?;
	fs::write(dir.path().join("src/routes.ts"), ROUTES)?;
	let engine = open(dir.path())?;
	let values = name();
	engine.apply(engine.plan(&PieceRequest {
		pack,
		piece: "page",
		inputs: &values,
		variant: None,
		target: None,
		granted: None,
		force: false,
	})?)?;
	Ok((dir, engine))
}

fn update(pack: &Pack, on_conflict: OnConflict) -> UpdateRequest<'_> {
	UpdateRequest {
		pack,
		granted: None,
		on_conflict,
		force: false,
		migrate: true,
	}
}

fn read(dir: &tempfile::TempDir, path: &str) -> Result<String, Box<dyn Error>> {
	Ok(fs::read_to_string(dir.path().join(path))?)
}

#[test]
fn update_should_take_the_new_version_of_unedited_files() -> TestResult {
	let (old, new) = (
		page("1.0.0", ORIGINAL)?,
		page("1.1.0", "one\ntwo\nthree\nfour\nFIVE\n")?,
	);
	let (dir, engine) = generated(&old)?;

	let plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	assert_eq!(plan.summary(), "update site 1.0.0 → 1.1.0");
	engine.apply(plan)?;

	assert_eq!(read(&dir, "src/pages/home.ts")?, "one\ntwo\nthree\nfour\nFIVE\n");
	let lock = engine.lockfile()?;
	assert_eq!(lock.packs[0].version.to_string(), "1.1.0");
	assert_eq!(lock.instances[0].pack_version.to_string(), "1.1.0");
	assert!(engine.status()?.iter().all(|status| status.state == FileState::Clean));
	Ok(())
}

#[test]
fn update_should_merge_edits_on_other_lines() -> TestResult {
	let (old, new) = (
		page("1.0.0", ORIGINAL)?,
		page("1.1.0", "one\ntwo\nthree\nfour\nFIVE\n")?,
	);
	let (dir, engine) = generated(&old)?;
	fs::write(dir.path().join("src/pages/home.ts"), "ONE\ntwo\nthree\nfour\nfive\n")?;

	engine.apply(engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?)?;

	assert_eq!(read(&dir, "src/pages/home.ts")?, "ONE\ntwo\nthree\nfour\nFIVE\n");
	assert_eq!(engine.status()?[0].state, FileState::Modified);
	Ok(())
}

#[test]
fn update_should_refuse_conflicts_by_default() -> TestResult {
	let (old, new) = (
		page("1.0.0", ORIGINAL)?,
		page("1.1.0", "one\nTWO\nthree\nfour\nfive\n")?,
	);
	let (dir, engine) = generated(&old)?;
	fs::write(dir.path().join("src/pages/home.ts"), "one\nmine\nthree\nfour\nfive\n")?;

	let result = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..]);

	let Err(EngineError::Rejected(rejected)) = result else {
		return Err(format!("expected a rejected plan, got {result:?}").into());
	};
	assert_eq!(rejected.problems[0].path.as_str(), "src/pages/home.ts");
	assert_eq!(rejected.problems[0].kind, ProblemKind::Conflict);
	Ok(())
}

#[test]
fn update_should_write_conflict_markers_when_asked() -> TestResult {
	let (old, new) = (
		page("1.0.0", ORIGINAL)?,
		page("1.1.0", "one\nTWO\nthree\nfour\nfive\n")?,
	);
	let (dir, engine) = generated(&old)?;
	fs::write(dir.path().join("src/pages/home.ts"), "one\nmine\nthree\nfour\nfive\n")?;

	let plan = engine.plan_update(&update(&new, OnConflict::Markers), &[old][..])?;
	assert_eq!(plan.changes()[0].conflicts, 1);
	engine.apply(plan)?;

	assert_eq!(
		read(&dir, "src/pages/home.ts")?,
		"one\n<<<<<<< yours\nmine\n||||||| original\ntwo\n=======\nTWO\n>>>>>>> site 1.1.0\nthree\nfour\nfive\n"
	);
	Ok(())
}

#[test]
fn update_should_resolve_conflicts_with_ours_or_theirs() -> TestResult {
	let (old, new) = (
		page("1.0.0", ORIGINAL)?,
		page("1.1.0", "one\nTWO\nthree\nfour\nFIVE\n")?,
	);
	let edited = "one\nmine\nthree\nfour\nfive\n";
	let (ours_dir, ours) = generated(&old)?;
	let (theirs_dir, theirs) = generated(&old)?;
	fs::write(ours_dir.path().join("src/pages/home.ts"), edited)?;
	fs::write(theirs_dir.path().join("src/pages/home.ts"), edited)?;

	ours.apply(ours.plan_update(&update(&new, OnConflict::Ours), std::slice::from_ref(&old))?)?;
	theirs.apply(theirs.plan_update(&update(&new, OnConflict::Theirs), &[old][..])?)?;

	assert_eq!(read(&ours_dir, "src/pages/home.ts")?, "one\nmine\nthree\nfour\nFIVE\n");
	assert_eq!(read(&theirs_dir, "src/pages/home.ts")?, "one\nTWO\nthree\nfour\nFIVE\n");
	Ok(())
}

#[test]
fn update_should_rebuild_the_original_from_the_old_pack_when_the_cache_is_gone() -> TestResult {
	let (old, new) = (
		page("1.0.0", ORIGINAL)?,
		page("1.1.0", "one\ntwo\nthree\nfour\nFIVE\n")?,
	);
	let (dir, engine) = generated(&old)?;
	fs::write(dir.path().join("src/pages/home.ts"), "ONE\ntwo\nthree\nfour\nfive\n")?;
	fs::remove_dir_all(dir.path().join(".tfverse/objects"))?;

	let without = engine.plan_update(&update(&new, OnConflict::Refuse), &[] as &[Pack]);
	engine.apply(engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?)?;

	let Err(EngineError::Rejected(rejected)) = without else {
		return Err(format!("expected a rejected plan, got {without:?}").into());
	};
	assert!(rejected.problems[0].message.contains("not available"), "{rejected}");
	assert_eq!(read(&dir, "src/pages/home.ts")?, "ONE\ntwo\nthree\nfour\nFIVE\n");
	Ok(())
}

#[test]
fn update_should_remove_files_the_new_version_dropped_and_undo_should_restore_them() -> TestResult {
	let old = pack("1.0.0", &[("my-page.ts", ORIGINAL), ("my-page.css", "a {}\n")], &[])?;
	let new = page("1.1.0", ORIGINAL)?;
	let (dir, engine) = generated(&old)?;

	let plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	let deleted: Vec<&str> = plan
		.changes()
		.iter()
		.filter(|change| change.action == Action::Delete)
		.map(|change| change.path.as_str())
		.collect();
	assert_eq!(deleted, ["src/pages/home.css"]);
	engine.apply(plan)?;
	assert!(!dir.path().join("src/pages/home.css").exists());

	engine.undo(false)?;
	assert_eq!(read(&dir, "src/pages/home.css")?, "a {}\n");
	assert_eq!(engine.lockfile()?.packs[0].version.to_string(), "1.0.0");
	Ok(())
}

#[test]
fn update_should_keep_an_edited_file_the_new_version_dropped() -> TestResult {
	let old = pack("1.0.0", &[("my-page.ts", ORIGINAL), ("my-page.css", "a {}\n")], &[])?;
	let new = page("1.1.0", ORIGINAL)?;
	let (dir, engine) = generated(&old)?;
	fs::write(dir.path().join("src/pages/home.css"), "a { color: red }\n")?;

	let plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	assert_eq!(plan.notes()[0].path.as_str(), "src/pages/home.css");
	engine.apply(plan)?;

	assert_eq!(read(&dir, "src/pages/home.css")?, "a { color: red }\n");
	assert!(
		engine
			.status()?
			.iter()
			.all(|status| status.path.as_str() != "src/pages/home.css")
	);
	Ok(())
}

#[test]
fn update_should_create_files_the_new_version_added() -> TestResult {
	let old = page("1.0.0", ORIGINAL)?;
	let new = pack("1.1.0", &[("my-page.ts", ORIGINAL), ("my-page.test.ts", "test\n")], &[])?;
	let (dir, engine) = generated(&old)?;

	engine.apply(engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?)?;

	assert_eq!(read(&dir, "src/pages/home.test.ts")?, "test\n");
	assert_eq!(engine.lockfile()?.instances[0].files.len(), 2);
	Ok(())
}

#[test]
fn update_should_leave_a_file_the_user_deleted() -> TestResult {
	let (old, new) = (page("1.0.0", ORIGINAL)?, page("1.1.0", "changed\n")?);
	let (dir, engine) = generated(&old)?;
	fs::remove_file(dir.path().join("src/pages/home.ts"))?;

	let plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	assert!(plan.notes()[0].message.contains("stays deleted"));
	engine.apply(plan)?;

	assert!(!dir.path().join("src/pages/home.ts").exists());
	Ok(())
}

#[test]
fn update_should_add_pieces_the_new_version_requires() -> TestResult {
	let old = page("1.0.0", ORIGINAL)?;
	let mut files = BTreeMap::new();
	for (path, text) in [
		(
			"pack.toml",
			"schema_version = 1\nname = \"site\"\nversion = \"1.1.0\"\n[permissions]\nwrite = [\"src/**\"]\n",
		),
		(
			"pieces/page/piece.toml",
			"schema_version = 1\nname = \"page\"\ntarget = \"src/pages\"\nrequires = [\"layout\"]\n[inputs.name]\ntype = \"string\"\nexample = \"MyPage\"\n",
		),
		("pieces/page/files/my-page.ts", ORIGINAL),
		(
			"pieces/layout/piece.toml",
			"schema_version = 1\nname = \"layout\"\ntarget = \"src\"\n",
		),
		("pieces/layout/files/layout.ts", "layout\n"),
	] {
		files.insert(path.to_owned(), text.as_bytes().to_vec());
	}
	let new = Pack::from_files(files)?;
	let (dir, engine) = generated(&old)?;

	engine.apply(engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?)?;

	assert_eq!(read(&dir, "src/layout.ts")?, "layout\n");
	assert_eq!(engine.lockfile()?.instances.len(), 2);
	Ok(())
}

#[test]
fn update_should_replace_the_regions_old_patches_inserted() -> TestResult {
	let patch = |insert: &str| {
		format!(
			"schema_version = 1\nname = \"page\"\ntarget = \"src/pages\"\n[inputs.name]\ntype = \"string\"\nexample = \"MyPage\"\n\
			[[patches]]\nkind = \"anchor\"\nfile = \"src/routes.ts\"\nanchor = \"// @tfverse:routes\"\nposition = \"before\"\ninsert = \"{insert}\"\n"
		)
	};
	let old_piece = patch("{ path: '/my-page' },");
	let new_piece = patch("{ path: '/my-page', component: MyPage },");
	let old = pack(
		"1.0.0",
		&[("my-page.ts", ORIGINAL)],
		&[("pieces/page/piece.toml", &old_piece)],
	)?;
	let new = pack(
		"1.1.0",
		&[("my-page.ts", ORIGINAL)],
		&[("pieces/page/piece.toml", &new_piece)],
	)?;
	let (dir, engine) = generated(&old)?;
	assert!(read(&dir, "src/routes.ts")?.contains("{ path: '/home' },"));

	engine.apply(engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?)?;

	assert_eq!(
		read(&dir, "src/routes.ts")?,
		"export const routes = [\n\t{ path: '/home', component: Home },\n\t// @tfverse:routes\n];\n"
	);
	Ok(())
}

#[test]
fn update_should_refuse_a_pack_the_project_does_not_use() -> TestResult {
	let dir = tempfile::tempdir()?;
	let engine = open(dir.path())?;
	let new = page("1.1.0", ORIGINAL)?;

	let result = engine.plan_update(&update(&new, OnConflict::Refuse), &[] as &[Pack]);

	assert!(matches!(result, Err(EngineError::NotInstalled(_))), "{result:?}");
	Ok(())
}

const LONG: &str = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";

#[test]
fn select_should_apply_only_the_kept_hunks_of_an_update() -> TestResult {
	let (old, new) = (
		page("1.0.0", LONG)?,
		page("1.1.0", "one\n2\n3\n4\n5\n6\n7\n8\n9\nten\n")?,
	);
	let (dir, engine) = generated(&old)?;
	let mut plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	let hunks = plan.changes()[0].hunks();
	assert_eq!(hunks.len(), 2);
	assert_eq!(hunks[1].lines, " 7\n 8\n 9\n-10\n+ten\n");

	let path = RelPath::new("src/pages/home.ts")?;
	let selection = Selection::from([(path, Pick::Hunks([1].into()))]);
	plan.select(&selection)?;
	engine.apply(plan)?;

	assert_eq!(read(&dir, "src/pages/home.ts")?, "1\n2\n3\n4\n5\n6\n7\n8\n9\nten\n");
	assert_eq!(engine.status()?[0].state, FileState::Modified);
	Ok(())
}

#[test]
fn a_skipped_hunk_should_stay_skipped_in_the_next_update() -> TestResult {
	let v1 = page("1.0.0", LONG)?;
	let v2 = page("1.1.0", "one\n2\n3\n4\n5\n6\n7\n8\n9\nten\n")?;
	let v3 = page("1.2.0", "one\n2\n3\n4\nfive\n6\n7\n8\n9\nten\n")?;
	let (dir, engine) = generated(&v1)?;
	let selection = Selection::from([(RelPath::new("src/pages/home.ts")?, Pick::Hunks([1].into()))]);
	let mut plan = engine.plan_update(&update(&v2, OnConflict::Refuse), std::slice::from_ref(&v1))?;
	plan.select(&selection)?;
	engine.apply(plan)?;

	engine.apply(engine.plan_update(&update(&v3, OnConflict::Refuse), &[v1, v2][..])?)?;

	assert_eq!(read(&dir, "src/pages/home.ts")?, "1\n2\n3\n4\nfive\n6\n7\n8\n9\nten\n");
	Ok(())
}

#[test]
fn a_skipped_new_file_should_not_be_written_or_recorded() -> TestResult {
	let dir = tempfile::tempdir()?;
	let engine = open(dir.path())?;
	let pack = pack("1.0.0", &[("my-page.ts", ORIGINAL), ("my-page.css", "a {}\n")], &[])?;
	let values = name();
	let mut plan = engine.plan(&PieceRequest {
		pack: &pack,
		piece: "page",
		inputs: &values,
		variant: None,
		target: None,
		granted: None,
		force: false,
	})?;
	let css = RelPath::new("src/pages/home.css")?;
	assert_eq!(
		plan
			.changes()
			.iter()
			.find(|change| change.path == css)
			.map(|change| change.hunks().len()),
		Some(1)
	);

	plan.select(&Selection::from([(css, Pick::None)]))?;
	engine.apply(plan)?;

	assert!(!dir.path().join("src/pages/home.css").exists());
	assert!(dir.path().join("src/pages/home.ts").exists());
	assert_eq!(engine.lockfile()?.instances[0].files.len(), 1);
	Ok(())
}

#[test]
fn select_should_reject_paths_and_hunks_the_plan_does_not_have() -> TestResult {
	let (old, new) = (
		page("1.0.0", LONG)?,
		page("1.1.0", "one\n2\n3\n4\n5\n6\n7\n8\n9\n10\n")?,
	);
	let (_dir, engine) = generated(&old)?;
	let mut plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	let path = RelPath::new("src/pages/home.ts")?;
	let diff = plan.changes()[0].diff.clone();

	let unknown_path = plan.select(&Selection::from([(RelPath::new("src/other.ts")?, Pick::All)]));
	let unknown_hunk = plan.select(&Selection::from([(path, Pick::Hunks([3].into()))]));

	assert!(matches!(unknown_path, Err(SelectError::UnknownPath(_))));
	assert!(matches!(unknown_hunk, Err(SelectError::UnknownHunk { index: 3, .. })));
	assert_eq!(plan.changes().len(), 1);
	assert_eq!(plan.changes()[0].diff, diff);
	Ok(())
}

#[test]
fn update_should_refuse_a_version_signed_by_another_key() -> TestResult {
	let old = page("1.0.0", LONG)?.with_signer("1C3B7AE36BE79191");
	let (_dir, engine) = generated(&old)?;
	assert_eq!(engine.lockfile()?.packs[0].signer.as_deref(), Some("1C3B7AE36BE79191"));
	let rotated = page("1.1.0", LONG)?.with_signer("0000000000000001");
	let unsigned = page("1.1.0", LONG)?;
	let packs = [old];

	let rotated = engine.plan_update(&update(&rotated, OnConflict::Refuse), &packs[..]);
	let unsigned = engine.plan_update(&update(&unsigned, OnConflict::Refuse), &packs[..]);

	assert!(
		matches!(rotated, Err(EngineError::SignerChanged { actual: Some(_), .. })),
		"{rotated:?}"
	);
	assert!(
		matches!(unsigned, Err(EngineError::SignerChanged { actual: None, .. })),
		"{unsigned:?}"
	);
	Ok(())
}

/// A pack.toml for `version` with `migrations` appended.
fn migrating(version: &str, migrations: &str) -> String {
	format!(
		"schema_version = 1\nname = \"site\"\nversion = \"{version}\"\n[permissions]\nwrite = [\"src/**\"]\n{migrations}"
	)
}

const RENAME: &str = "[[migrations]]\nto = \"1.1.0\"\ndescription = \"Rename OldName to NewName\"\nfiles = [\"src/**/*.ts\"]\nrename = { from = \"OldName\", to = \"NewName\" }\n";

#[test]
fn update_should_run_the_migrations_it_crosses_on_project_code() -> TestResult {
	let old = page("1.0.0", ORIGINAL)?;
	let new = pack(
		"1.1.0",
		&[("my-page.ts", ORIGINAL)],
		&[("pack.toml", &migrating("1.1.0", RENAME))],
	)?;
	let (dir, engine) = generated(&old)?;
	fs::write(
		dir.path().join("src/app.ts"),
		"import { OldName } from './x'\nconst y = OldNameLike\n",
	)?;
	fs::write(dir.path().join("src/app.css"), "OldName {}\n")?;

	let plan = engine.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?;
	assert!(
		plan
			.notes()
			.iter()
			.any(|note| note.message.contains("Rename OldName to NewName (1 file(s) changed)"))
	);
	engine.apply(plan)?;

	assert_eq!(
		read(&dir, "src/app.ts")?,
		"import { NewName } from './x'\nconst y = OldNameLike\n"
	);
	assert_eq!(read(&dir, "src/app.css")?, "OldName {}\n");
	Ok(())
}

#[test]
fn update_should_skip_migrations_it_does_not_cross_or_was_told_not_to_run() -> TestResult {
	let v110 = pack(
		"1.1.0",
		&[("my-page.ts", ORIGINAL)],
		&[("pack.toml", &migrating("1.1.0", RENAME))],
	)?;
	let v120 = pack(
		"1.2.0",
		&[("my-page.ts", ORIGINAL)],
		&[("pack.toml", &migrating("1.2.0", RENAME))],
	)?;
	let (dir, engine) = generated(&v110)?;
	fs::write(dir.path().join("src/app.ts"), "OldName\n")?;

	let crossed_before = engine.plan_update(&update(&v120, OnConflict::Refuse), std::slice::from_ref(&v110))?;
	let (dir_off, engine_off) = generated(&page("1.0.0", ORIGINAL)?)?;
	fs::write(dir_off.path().join("src/app.ts"), "OldName\n")?;
	let off = engine_off.plan_update(
		&UpdateRequest {
			migrate: false,
			..update(&v110, OnConflict::Refuse)
		},
		&[] as &[Pack],
	)?;

	assert!(
		crossed_before
			.changes()
			.iter()
			.all(|change| change.path.as_str() != "src/app.ts")
	);
	assert!(off.changes().iter().all(|change| change.path.as_str() != "src/app.ts"));
	assert_eq!(read(&dir, "src/app.ts")?, "OldName\n");
	Ok(())
}

/// Stands in for ast-grep: rewrites `cx(` to `cn(`.
#[derive(Debug)]
struct FakeAstGrep;

impl AstGrep for FakeAstGrep {
	fn rewrite(&self, rule: &str, text: &str) -> Result<Option<String>, MigrateError> {
		assert!(rule.contains("id: cx-to-cn"));
		Ok(text.contains("cx(").then(|| text.replace("cx(", "cn(")))
	}
}

#[test]
fn update_should_run_ast_grep_rules_shipped_in_the_pack() -> TestResult {
	let migration = "[[migrations]]\nto = \"1.1.0\"\ndescription = \"cx is now cn\"\nfiles = [\"src/**/*.ts\"]\nast_grep = \"migrations/1.1.0.yml\"\n";
	let old = page("1.0.0", ORIGINAL)?;
	let new = pack(
		"1.1.0",
		&[("my-page.ts", ORIGINAL)],
		&[
			("pack.toml", &migrating("1.1.0", migration)),
			(
				"migrations/1.1.0.yml",
				"id: cx-to-cn\nlanguage: ts\nrule: { pattern: cx($$$A) }\nfix: cn($$$A)\n",
			),
		],
	)?;
	let (dir, _) = generated(&old)?;
	fs::write(dir.path().join("src/app.ts"), "const a = cx(b)\n")?;
	let faked = Engine::builder()
		.user_config(UserConfig::default())
		.ast_grep(FakeAstGrep)
		.open(dir.path())?;
	let missing = Engine::builder()
		.user_config(UserConfig {
			ast_grep: Some(tfverse_core::workspace::ExternalPatcher {
				command: vec!["tfverse-no-such-ast-grep".to_owned()],
			}),
			..UserConfig::default()
		})
		.open(dir.path())?;

	let without = missing.plan_update(&update(&new, OnConflict::Refuse), std::slice::from_ref(&old));
	faked.apply(faked.plan_update(&update(&new, OnConflict::Refuse), &[old][..])?)?;

	let Err(EngineError::Rejected(rejected)) = without else {
		return Err(format!("expected a rejected plan, got {without:?}").into());
	};
	assert_eq!(rejected.problems[0].kind, ProblemKind::Unavailable);
	assert_eq!(read(&dir, "src/app.ts")?, "const a = cn(b)\n");
	Ok(())
}

#[test]
fn pack_should_reject_invalid_migrations() {
	let invalid = [
		"[[migrations]]\nto = \"9.0.0\"\ndescription = \"x\"\nfiles = [\"src/**\"]\nrename = { from = \"a\", to = \"b\" }\n",
		"[[migrations]]\nto = \"1.0.0\"\ndescription = \"x\"\nfiles = [\"src/**\"]\n",
		"[[migrations]]\nto = \"1.0.0\"\ndescription = \"x\"\nfiles = [\"src/**\"]\nast_grep = \"pieces/page/piece.toml\"\n",
		"[[migrations]]\nto = \"1.0.0\"\ndescription = \"x\"\nfiles = [\"src/**\"]\nrename = { from = \"a.b\", to = \"c\" }\n",
	];

	for migrations in invalid {
		let result = pack(
			"1.0.0",
			&[("my-page.ts", ORIGINAL)],
			&[("pack.toml", &migrating("1.0.0", migrations))],
		);
		assert!(result.is_err(), "{migrations}");
	}
}
