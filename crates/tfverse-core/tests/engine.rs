//! End-to-end engine tests against the fixture pack.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tfverse_core::engine::{FileState, LineOrigin};
use tfverse_core::history::HistoryError;
use tfverse_core::model::{Glob, Permissions};
use tfverse_core::patch::{Patch, PatchError, PatchKind, Patched, Patcher};
use tfverse_core::plan::{Action, ProblemKind};
use tfverse_core::workspace::ExternalPatcher;
use tfverse_core::{Engine, EngineError, Event, MemoryFs, Pack, PieceRequest, RelPath, UserConfig, Value, Values};

type TestResult = Result<(), Box<dyn Error>>;

const ROUTES: &str = "import { Home } from './pages/home';\n\nexport const routes = [\n\t{ path: '/', component: Home },\n\t// @tfverse:routes\n];\n";

fn pack() -> Result<Pack, Box<dyn Error>> {
	Ok(Pack::load(
		&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-pack"),
	)?)
}

fn project(framework: &str) -> Result<tempfile::TempDir, Box<dyn Error>> {
	let dir = tempfile::tempdir()?;
	let package = format!(r#"{{"dependencies": {{"{framework}": "1"}}, "devDependencies": {{"typescript": "6"}}}}"#);
	fs::write(dir.path().join("package.json"), package)?;
	fs::create_dir(dir.path().join("src"))?;
	fs::write(dir.path().join("src/routes.ts"), ROUTES)?;
	Ok(dir)
}

fn open(root: &Path) -> Result<Engine, EngineError> {
	Engine::builder().user_config(UserConfig::default()).open(root)
}

fn inputs(pairs: &[(&str, Value)]) -> Values {
	pairs
		.iter()
		.map(|(name, value)| ((*name).to_owned(), value.clone()))
		.collect()
}

fn text(value: &str) -> Value {
	Value::String(value.to_owned())
}

fn request<'a>(pack: &'a Pack, piece: &'a str, inputs: &'a Values) -> PieceRequest<'a> {
	PieceRequest {
		pack,
		piece,
		inputs,
		variant: None,
		target: None,
		granted: None,
		force: false,
	}
}

fn tree(root: &Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
	fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, String>) -> Result<(), Box<dyn Error>> {
		for entry in fs::read_dir(dir)? {
			let path: PathBuf = entry?.path();
			let relative = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
			if relative.starts_with(".tfverse") {
				continue;
			}
			if path.is_dir() {
				walk(root, &path, files)?;
			} else {
				files.insert(relative, fs::read_to_string(&path)?);
			}
		}
		Ok(())
	}
	let mut files = BTreeMap::new();
	walk(root, root, &mut files)?;
	Ok(files)
}

#[test]
fn react_component_plan_should_match_snapshot() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("UserCard")), ("styled", Value::Bool(true))]);

	let plan = engine.plan(&request(&pack, "component", &values))?;

	insta::assert_yaml_snapshot!(plan.report());
	Ok(())
}

#[test]
fn react_component_should_render_real_code() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("UserCard")), ("styled", Value::Bool(true))]);

	engine.apply(engine.plan(&request(&pack, "component", &values))?)?;

	insta::assert_yaml_snapshot!(tree(dir.path())?);
	Ok(())
}

#[test]
fn svelte_project_should_pick_svelte_variant() -> TestResult {
	let (pack, dir) = (pack()?, project("svelte")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("user card"))]);

	engine.apply(engine.plan(&request(&pack, "component", &values))?)?;

	insta::assert_snapshot!(fs::read_to_string(dir.path().join("src/components/user-card.svelte"))?);
	Ok(())
}

#[test]
fn route_should_patch_existing_routes_file() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("Settings"))]);

	engine.apply(engine.plan(&request(&pack, "route", &values))?)?;

	insta::assert_snapshot!(fs::read_to_string(dir.path().join("src/routes.ts"))?);
	Ok(())
}

#[test]
fn second_plan_should_change_nothing() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("Settings"))]);
	engine.apply(engine.plan(&request(&pack, "route", &values))?)?;

	let again = engine.plan(&request(&pack, "route", &values))?;

	assert!(again.changes().iter().all(|change| change.action == Action::Unchanged));
	Ok(())
}

#[test]
fn undo_should_restore_the_project_exactly() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let before = tree(dir.path())?;
	let engine = open(dir.path())?;
	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?)?;

	engine.undo(false)?;

	assert_eq!(tree(dir.path())?, before);
	Ok(())
}

#[test]
fn undo_should_refuse_to_discard_later_edits() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?)?;
	fs::write(dir.path().join("src/pages/settings.tsx"), "edited\n")?;

	let result = engine.undo(false);

	assert!(matches!(
		result,
		Err(EngineError::History(HistoryError::Diverged { .. }))
	));
	Ok(())
}

#[test]
fn history_should_list_applied_transactions() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("A"))])))?)?;
	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("B"))])))?)?;
	engine.undo(false)?;

	let summaries: Vec<String> = engine.history()?.into_iter().map(|entry| entry.summary).collect();

	assert_eq!(summaries, ["add sample/route"]);
	Ok(())
}

#[test]
fn status_should_report_modified_and_missing_files() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("UserCard")), ("styled", Value::Bool(true))]);
	engine.apply(engine.plan(&request(&pack, "component", &values))?)?;
	fs::write(dir.path().join("src/components/user-card.tsx"), "changed\n")?;
	fs::remove_file(dir.path().join("src/components/user-card.module.css"))?;

	let states: Vec<(String, FileState)> = engine
		.status()?
		.into_iter()
		.map(|status| (status.path.to_string(), status.state))
		.collect();

	assert_eq!(
		states,
		[
			("src/components/user-card.module.css".to_owned(), FileState::Missing),
			("src/components/user-card.tsx".to_owned(), FileState::Modified)
		]
	);
	Ok(())
}

#[test]
fn why_should_tell_pack_lines_from_user_lines() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?)?;
	let page = dir.path().join("src/pages/settings.tsx");
	fs::write(&page, format!("// added by me\n{}", fs::read_to_string(&page)?))?;
	let path = RelPath::new("src/pages/settings.tsx")?;
	let packs = [pack];

	let user = engine.why(&path, 1, &packs[..])?.map(|provenance| provenance.line);
	let original = engine.why(&path, 2, &packs[..])?.map(|provenance| provenance.line);

	assert_eq!(
		(user, original),
		(Some(LineOrigin::User), Some(LineOrigin::Pack { original_line: 1 }))
	);
	Ok(())
}

#[test]
fn base_should_be_rebuilt_from_the_pack_when_cache_is_gone() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("UserCard")), ("styled", Value::Bool(true))]);
	engine.apply(engine.plan(&request(&pack, "component", &values))?)?;
	let path = RelPath::new("src/components/user-card.tsx")?;
	let generated = fs::read(dir.path().join(path.as_str()))?;
	fs::remove_dir_all(dir.path().join(".tfverse"))?;

	let packs = [pack];
	let base = engine.base(&path, &packs[..])?;

	assert_eq!(base, Some(generated));
	Ok(())
}

#[test]
fn plan_should_reject_writes_outside_granted_paths() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("UserCard"))]);
	let granted = Permissions {
		write: vec![Glob::new("src/pages/**")?],
		..Permissions::default()
	};

	let result = engine.plan(&PieceRequest {
		granted: Some(&granted),
		..request(&pack, "component", &values)
	});

	assert!(
		matches!(result, Err(EngineError::Rejected(rejected)) if rejected.problems[0].kind == ProblemKind::NotGranted)
	);
	Ok(())
}

#[test]
fn plan_should_refuse_to_overwrite_a_users_file() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	fs::create_dir_all(dir.path().join("src/components"))?;
	fs::write(dir.path().join("src/components/user-card.tsx"), "mine\n")?;
	let engine = open(dir.path())?;
	let values = inputs(&[("name", text("UserCard"))]);

	let result = engine.plan(&request(&pack, "component", &values));

	assert!(matches!(result, Err(EngineError::Rejected(rejected)) if rejected.problems[0].kind == ProblemKind::Conflict));
	Ok(())
}

#[test]
fn apply_should_refuse_a_plan_made_stale_by_an_edit() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	let plan = engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?;
	fs::write(
		dir.path().join("src/routes.ts"),
		"export const routes = [];\n// @tfverse:routes\n",
	)?;

	let result = engine.apply(plan);

	assert!(matches!(result, Err(EngineError::History(HistoryError::Changed(_)))));
	Ok(())
}

#[test]
fn lockfile_should_record_the_instance() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = open(dir.path())?;
	engine.apply(engine.plan(&request(&pack, "component", &inputs(&[("name", text("UserCard"))])))?)?;

	insta::assert_snapshot!(fs::read_to_string(dir.path().join("tfverse.lock"))?);
	Ok(())
}

#[cfg(unix)]
#[test]
fn user_configured_patcher_should_replace_the_builtin_one() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let script = r#"cat > /dev/null; printf '{"status":"changed","source":"// patched by my tool\\n// @tfverse:routes\\n","inserted":"x"}'"#;
	let mut user = UserConfig::default();
	user.patchers.insert(
		PatchKind::Import,
		ExternalPatcher {
			command: vec!["sh".to_owned(), "-c".to_owned(), script.to_owned()],
		},
	);
	let engine = Engine::builder().user_config(user).open(dir.path())?;

	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?)?;

	assert!(fs::read_to_string(dir.path().join("src/routes.ts"))?.starts_with("// patched by my tool\n"));
	Ok(())
}

#[test]
fn memory_project_should_plan_and_apply_without_touching_disk() -> TestResult {
	let pack = pack()?;
	let fs = Arc::new(MemoryFs::new());
	fs.insert(&RelPath::new("package.json")?, r#"{"dependencies": {"svelte": "5"}}"#);
	let engine = Engine::builder()
		.user_config(UserConfig::default())
		.open_fs(fs.clone())?;

	engine.apply(engine.plan(&request(&pack, "component", &inputs(&[("name", text("UserCard"))])))?)?;

	let files = fs.files();
	assert!(files.contains_key(&RelPath::new("src/components/user-card.svelte")?));
	assert!(files.contains_key(&RelPath::new("tfverse.lock")?));
	Ok(())
}

#[test]
fn sink_should_receive_write_and_apply_events() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let seen = Arc::new(Mutex::new(Vec::new()));
	let recorder = Arc::clone(&seen);
	let engine = Engine::builder()
		.user_config(UserConfig::default())
		.sink(move |event: &Event<'_>| {
			let label = match event {
				Event::Wrote(path) => format!("wrote {path}"),
				Event::Applied { transaction, .. } => format!("applied {transaction}"),
				_ => "other".to_owned(),
			};
			recorder.lock().map(|mut events| events.push(label)).ok();
		})
		.open(dir.path())?;

	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?)?;

	let events = seen.lock().map_err(|_| "poisoned")?.clone();
	assert_eq!(events.last().map(String::as_str), Some("applied 1"));
	assert!(events.contains(&"wrote src/routes.ts".to_owned()));
	Ok(())
}

struct Shout;

impl Patcher for Shout {
	fn apply(&self, text: &str, _patch: &Patch) -> Result<Patched, PatchError> {
		Ok(Patched::Changed {
			text: format!("// SHOUT\n{text}"),
			inserted: "// SHOUT\n".to_owned(),
		})
	}
}

#[test]
fn builder_patcher_should_replace_the_builtin_one() -> TestResult {
	let (pack, dir) = (pack()?, project("react")?);
	let engine = Engine::builder()
		.user_config(UserConfig::default())
		.patcher(PatchKind::Import, Shout)
		.open(dir.path())?;

	engine.apply(engine.plan(&request(&pack, "route", &inputs(&[("name", text("Settings"))])))?)?;

	assert!(fs::read_to_string(dir.path().join("src/routes.ts"))?.starts_with("// SHOUT\n"));
	Ok(())
}

/// A pack whose `resource` piece requires `error` and imports `zod`.
fn layered_pack(dependencies_allowed: bool) -> Result<Pack, Box<dyn Error>> {
	let pack_toml = format!(
		"schema_version = 1\nname = \"layered\"\nversion = \"1.0.0\"\n[permissions]\nwrite = [\"src/**\"]\ndependencies = {dependencies_allowed}\n"
	);
	let error = "schema_version = 1\nname = \"error\"\ntarget = \"src/shared\"\n";
	let resource = "schema_version = 1\nname = \"resource\"\ntarget = \"src/modules\"\nrequires = [\"error\"]\n\
		[inputs.name]\ntype = \"string\"\nexample = \"MyThing\"\n\
		[[dependencies]]\necosystem = \"npm\"\nname = \"zod\"\nversion = \"^4.1\"\n";
	let files = BTreeMap::from([
		("pack.toml".to_owned(), pack_toml.into_bytes()),
		("pieces/error/piece.toml".to_owned(), error.as_bytes().to_vec()),
		(
			"pieces/error/files/errors.ts".to_owned(),
			b"export class AppError extends Error {}\n".to_vec(),
		),
		("pieces/resource/piece.toml".to_owned(), resource.as_bytes().to_vec()),
		(
			"pieces/resource/files/my-thing.ts".to_owned(),
			b"export const myThing = 1;\n".to_vec(),
		),
	]);
	Ok(Pack::from_files(files)?)
}

fn memory_engine() -> Result<(Arc<MemoryFs>, Engine), Box<dyn Error>> {
	let fs = Arc::new(MemoryFs::new());
	let engine = Engine::builder()
		.user_config(UserConfig::default())
		.open_fs(fs.clone())?;
	Ok((fs, engine))
}

#[test]
fn plan_should_add_required_pieces_the_project_lacks() -> TestResult {
	let pack = layered_pack(true)?;
	let (fs, engine) = memory_engine()?;
	let values = inputs(&[("name", text("Invoice"))]);

	let plan = engine.plan(&request(&pack, "resource", &values))?;
	assert_eq!(plan.summary(), "add layered/resource (with error)");
	engine.apply(plan)?;

	let files = fs.files();
	assert!(files.contains_key(&RelPath::new("src/shared/errors.ts")?));
	assert!(files.contains_key(&RelPath::new("src/modules/invoice.ts")?));
	Ok(())
}

#[test]
fn plan_should_skip_required_pieces_already_in_the_lockfile() -> TestResult {
	let pack = layered_pack(true)?;
	let (_fs, engine) = memory_engine()?;
	engine.apply(engine.plan(&request(&pack, "resource", &inputs(&[("name", text("Invoice"))])))?)?;

	let plan = engine.plan(&request(&pack, "resource", &inputs(&[("name", text("Customer"))])))?;

	assert_eq!(plan.summary(), "add layered/resource");
	assert_eq!(plan.instances().len(), 1);
	Ok(())
}

#[test]
fn plan_should_list_dependencies_without_installing_them() -> TestResult {
	let pack = layered_pack(true)?;
	let (fs, engine) = memory_engine()?;

	let plan = engine.plan(&request(&pack, "resource", &inputs(&[("name", text("Invoice"))])))?;

	assert_eq!(
		plan.dependencies().iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
		["zod"]
	);
	engine.apply(plan)?;
	assert!(!fs.files().contains_key(&RelPath::new("package.json")?));
	Ok(())
}

#[test]
fn plan_should_reject_dependencies_the_user_did_not_grant() -> TestResult {
	let pack = layered_pack(true)?;
	let (_fs, engine) = memory_engine()?;
	let values = inputs(&[("name", text("Invoice"))]);
	let granted = Permissions {
		write: vec![Glob::new("src/**")?],
		..Permissions::default()
	};

	let result = engine.plan(&PieceRequest {
		granted: Some(&granted),
		..request(&pack, "resource", &values)
	});

	let Err(EngineError::Rejected(rejected)) = result else {
		return Err("expected a rejected plan".into());
	};
	assert_eq!(rejected.problems[0].path.as_str(), "package.json");
	assert_eq!(rejected.problems[0].kind, ProblemKind::NotGranted);
	Ok(())
}

#[test]
fn pack_should_require_permission_to_declare_dependencies() {
	let error = layered_pack(false).map(|_| ()).unwrap_err().to_string();

	assert!(error.contains("permissions.dependencies"), "{error}");
}

#[test]
fn plan_should_refuse_a_pack_changed_without_a_new_version() -> TestResult {
	let (fs, engine) = memory_engine()?;
	let pack = layered_pack(true)?;
	engine.apply(engine.plan(&request(&pack, "error", &Values::new()))?)?;
	let mut files = BTreeMap::new();
	for (path, bytes) in [
		(
			"pack.toml",
			"schema_version = 1\nname = \"layered\"\nversion = \"1.0.0\"\n[permissions]\nwrite = [\"src/**\"]\n",
		),
		(
			"pieces/error/piece.toml",
			"schema_version = 1\nname = \"error\"\ntarget = \"src/shared\"\n",
		),
		(
			"pieces/error/files/errors.ts",
			"export class Tampered extends Error {}\n",
		),
	] {
		files.insert(path.to_owned(), bytes.as_bytes().to_vec());
	}
	let tampered = Pack::from_files(files)?;

	let result = engine.plan(&request(&tampered, "error", &Values::new()));

	assert!(matches!(result, Err(EngineError::PackChanged { .. })), "{result:?}");
	assert!(fs.files().contains_key(&RelPath::new("tfverse.lock")?));
	Ok(())
}
