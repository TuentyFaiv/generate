//! Adopting existing code and capturing project code as a piece.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use tfverse_core::engine::{FileState, UpdateRequest};
use tfverse_core::plan::{Action, OnConflict};
use tfverse_core::{CaptureRequest, Engine, EngineError, Pack, PieceRequest, RelPath, UserConfig, Value, Values};

type TestResult = Result<(), Box<dyn Error>>;

fn page(version: &str, text: &str) -> Result<Pack, Box<dyn Error>> {
	Ok(Pack::from_files(BTreeMap::from([
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
		("pieces/page/files/my-page.ts".to_owned(), text.as_bytes().to_vec()),
	]))?)
}

fn open(root: &Path) -> Result<Engine, EngineError> {
	Engine::builder().user_config(UserConfig::default()).open(root)
}

fn named(name: &str) -> Values {
	Values::from([("name".to_owned(), Value::String(name.to_owned()))])
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

#[test]
fn adopt_should_record_existing_code_without_changing_it_and_then_merge_updates() -> TestResult {
	let v1 = page(
		"1.0.0",
		"export const page = 'MyPage'\nexport const a = 1\nexport const b = 2\nexport const c = 3\n",
	)?;
	let v2 = page(
		"1.1.0",
		"export const page = 'MyPage'\nexport const a = 1\nexport const b = 2\nexport const c = 30\n",
	)?;
	let dir = tempfile::tempdir()?;
	fs::create_dir_all(dir.path().join("src/pages"))?;
	let mine = "// mine\nexport const page = 'Home'\nexport const a = 1\nexport const b = 2\nexport const c = 3\n";
	fs::write(dir.path().join("src/pages/home.ts"), mine)?;
	let engine = open(dir.path())?;
	let values = named("Home");

	let plan = engine.plan_adopt(&request(&v1, "page", &values))?;
	assert_eq!(plan.summary(), "adopt site/page");
	assert!(plan.changes().is_empty(), "{:?}", plan.changes());
	assert_eq!(plan.notes()[0].path.as_str(), "src/pages/home.ts");
	engine.apply(plan)?;

	assert_eq!(fs::read_to_string(dir.path().join("src/pages/home.ts"))?, mine);
	assert_eq!(engine.status()?[0].state, FileState::Modified);
	engine.apply(engine.plan_update(
		&UpdateRequest {
			pack: &v2,
			granted: None,
			on_conflict: OnConflict::Refuse,
			force: false,
			migrate: true,
		},
		std::slice::from_ref(&v1),
	)?)?;
	assert_eq!(
		fs::read_to_string(dir.path().join("src/pages/home.ts"))?,
		"// mine\nexport const page = 'Home'\nexport const a = 1\nexport const b = 2\nexport const c = 30\n"
	);
	Ok(())
}

#[test]
fn adopt_should_create_the_files_the_project_lacks() -> TestResult {
	let dir = tempfile::tempdir()?;
	let engine = open(dir.path())?;
	let pack = page("1.0.0", "x\n")?;
	let values = named("Home");

	let plan = engine.plan_adopt(&request(&pack, "page", &values))?;

	assert_eq!(plan.changes()[0].action, Action::Create);
	Ok(())
}

#[test]
fn capture_should_make_a_piece_that_regenerates_the_code_under_any_name() -> TestResult {
	let dir = tempfile::tempdir()?;
	let card = "export function UserCard() {\n  return <div className=\"user-card\">userCard</div>\n}\n";
	fs::create_dir_all(dir.path().join("src/ui/user-card"))?;
	fs::write(dir.path().join("src/ui/user-card/user-card.tsx"), card)?;
	fs::write(
		dir.path().join("src/ui/user-card/index.ts"),
		"export * from './user-card'\n",
	)?;
	let engine = open(dir.path())?;
	let files = [
		RelPath::new("src/ui/user-card/index.ts")?,
		RelPath::new("src/ui/user-card/user-card.tsx")?,
	];

	let plan = engine.plan_capture(&CaptureRequest {
		pack_dir: &RelPath::new("packs/mine")?,
		pack: "mine",
		piece: "card",
		files: &files,
		example: Some("UserCard"),
		force: false,
	})?;
	assert!(
		plan
			.changes()
			.iter()
			.all(|change| change.path.as_str().starts_with("packs/mine/"))
	);
	engine.apply(plan)?;

	let pack = Pack::load(&dir.path().join("packs/mine"))?;
	let same = named("UserCard");
	let again = engine.plan(&request(&pack, "card", &same))?;
	assert!(
		again.changes().iter().all(|change| change.action == Action::Unchanged),
		"{:?}",
		again.changes()
	);
	let other = named("ProfileBadge");
	engine.apply(engine.plan(&request(&pack, "card", &other))?)?;
	assert_eq!(
		fs::read_to_string(dir.path().join("src/ui/profile-badge/profile-badge.tsx"))?,
		"export function ProfileBadge() {\n  return <div className=\"profile-badge\">profileBadge</div>\n}\n"
	);
	assert_eq!(
		fs::read_to_string(dir.path().join("src/ui/profile-badge/index.ts"))?,
		"export * from './profile-badge'\n"
	);
	Ok(())
}

#[test]
fn capture_should_refuse_missing_files_and_bad_names() -> TestResult {
	let dir = tempfile::tempdir()?;
	let engine = open(dir.path())?;
	let pack_dir = RelPath::new("packs/mine")?;
	let missing = [RelPath::new("src/nope.ts")?];
	let capture = |piece| CaptureRequest {
		pack_dir: &pack_dir,
		pack: "mine",
		piece,
		files: &missing,
		example: None,
		force: false,
	};

	assert!(matches!(
		engine.plan_capture(&capture("card")),
		Err(EngineError::Capture(_))
	));
	assert!(matches!(
		engine.plan_capture(&capture("Bad Name")),
		Err(EngineError::Capture(_))
	));
	Ok(())
}
