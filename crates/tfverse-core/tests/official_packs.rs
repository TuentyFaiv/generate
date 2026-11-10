//! Renders every piece of the official packs in `packs/`, in every variant.

use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tfverse_core::model::{Input, Value};
use tfverse_core::{Engine, MemoryFs, Pack, PieceRequest, RelPath, UserConfig, Values};

type TestResult = Result<(), Box<dyn Error>>;

/// Input values that share no word with any template example.
const INPUTS: [(&str, &str); 5] = [
	("name", "StatusBadge"),
	("plural", "StatusBadges"),
	("app", "Shop"),
	("package", "dev.acme.shop"),
	("module", "github.com/acme/shop"),
];

/// Lowercase spellings of an example that must not survive rendering, e.g. `requestid`, `request-id`.
fn spellings(example: &str) -> Vec<String> {
	let mut words: Vec<String> = Vec::new();
	for c in example.chars() {
		if !c.is_alphanumeric() {
			words.push(String::new());
		} else if c.is_uppercase() || words.is_empty() {
			words.push(c.to_lowercase().collect());
		} else if let Some(word) = words.last_mut() {
			word.push(c);
		}
	}
	words.retain(|word| !word.is_empty());
	["", "-", "_", " "]
		.iter()
		.map(|separator| words.join(separator))
		.collect()
}

fn packs_dir() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs")
}

fn pack(name: &str) -> Result<Pack, Box<dyn Error>> {
	Ok(Pack::load(&packs_dir().join(name))?)
}

/// Renders one variant into an empty project with the given stack and returns its files.
fn render(pack: &Pack, piece: &str, variant: &str, stack: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
	let fs = Arc::new(MemoryFs::new());
	fs.insert(
		&RelPath::new("tfverse.json")?,
		format!(r#"{{"stack": {stack}}}"#).as_str(),
	);
	let engine = Engine::builder()
		.user_config(UserConfig::default())
		.open_fs(fs.clone())?;
	let declared = &pack.piece(piece).ok_or("missing piece")?.manifest().inputs;
	let inputs: Values = INPUTS
		.iter()
		.filter(|(name, _)| declared.contains_key(*name))
		.map(|(name, value)| ((*name).to_owned(), Value::String((*value).to_owned())))
		.collect();

	engine.apply(engine.plan(&PieceRequest {
		pack,
		piece,
		inputs: &inputs,
		variant: Some(variant),
		target: None,
		granted: None,
		force: false,
	})?)?;

	Ok(
		fs.files()
			.into_iter()
			.map(|(path, bytes)| (path.to_string(), String::from_utf8_lossy(&bytes).into_owned()))
			.filter(|(path, _)| path != "tfverse.json" && path != "tfverse.lock" && !path.starts_with(".tfverse/"))
			.collect(),
	)
}

/// Every piece and variant of `name`, rendered with each stack, as a list of written paths.
fn rendered_paths(name: &str, stacks: &[(&str, &str)]) -> Result<String, Box<dyn Error>> {
	let pack = pack(name)?;
	let mut listing = String::new();
	for piece in pack.pieces() {
		let manifest = piece.manifest();
		let leftovers: Vec<String> = manifest
			.inputs
			.values()
			.filter_map(|input| match input {
				Input::String {
					example: Some(example), ..
				} => Some(spellings(example)),
				_ => None,
			})
			.flatten()
			.chain(["example".to_owned()])
			.collect();
		for variant in &manifest.variants {
			for (label, stack) in stacks {
				let files = render(&pack, manifest.name.as_str(), variant.name.as_str(), stack)?;
				writeln!(listing, "{} · {}{label}", manifest.name, variant.name)?;
				for (path, contents) in &files {
					writeln!(listing, "  {path}")?;
					let text = format!("{path}\n{contents}").to_lowercase();
					let leftover = leftovers.iter().find(|spelling| text.contains(spelling.as_str()));
					assert!(
						leftover.is_none(),
						"`{path}` of {} · {} still has {leftover:?}",
						manifest.name,
						variant.name
					);
					assert!(!contents.contains("@tfverse"), "`{path}` still has a directive");
				}
			}
		}
	}
	Ok(listing)
}

#[test]
fn ui_pack_should_render_every_variant_with_and_without_tailwind() -> TestResult {
	let listing = rendered_paths("ui", &[("", "{}"), (" (tailwind)", r#"{"styling": ["tailwind"]}"#)])?;

	insta::assert_snapshot!(listing);
	Ok(())
}

#[test]
fn backend_pack_should_render_every_variant_with_and_without_an_orm() -> TestResult {
	let orms = r#"{"database": ["drizzle", "prisma", "sqlx", "sqlalchemy", "gorm"]}"#;
	let listing = rendered_paths("backend", &[("", "{}"), (" (orm)", orms)])?;

	insta::assert_snapshot!(listing);
	Ok(())
}
