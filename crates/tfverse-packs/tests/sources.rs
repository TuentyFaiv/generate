//! Loading packs through the public API, with downloads served in memory.

use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, Mutex, PoisonError};

use tfverse_core::engine::UpdateRequest;
use tfverse_core::model::Digest;
use tfverse_core::plan::OnConflict;
use tfverse_core::workspace::{Lockfile, PackRecord, ProjectConfig};
use tfverse_core::{Engine, PieceRequest, UserConfig, Values};
use tfverse_packs::transport::{Fetch, FetchError, Header};
use tfverse_packs::{Cache, Resolver};

type TestResult = Result<(), Box<dyn Error>>;

const BUTTON: &str = r#"{"name":"button","registryDependencies":["utils"],"files":[{"path":"registry/default/ui/button.tsx","type":"registry:ui","content":"import { cn } from \"@/registry/default/lib/utils\"\nexport const label = 1\n"}],"type":"registry:ui"}"#;
const UTILS: &str = r#"{"name":"utils","files":[{"path":"registry/default/lib/utils.ts","type":"registry:lib","content":"export const cn = 1\n"}],"type":"registry:lib"}"#;
const BUTTON_URL: &str = "https://registry.example/r/button.json";

/// Serves files by the last part of their URL; they can change between requests.
#[derive(Debug, Default)]
struct Served(Mutex<BTreeMap<String, String>>);

impl Served {
	fn set(&self, name: &str, text: &str) {
		self
			.0
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.insert(name.to_owned(), text.to_owned());
	}
}

impl Fetch for Served {
	fn get(&self, url: &str, _headers: &[Header]) -> Result<Vec<u8>, FetchError> {
		let name = url.rsplit('/').next().unwrap_or_default();
		self
			.0
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.get(name)
			.map(|text| text.as_bytes().to_vec())
			.ok_or_else(|| FetchError {
				url: url.to_owned(),
				reason: "the server answered HTTP 404".to_owned(),
			})
	}
}

#[test]
fn a_registry_item_should_install_and_update_through_the_engine() -> TestResult {
	let served = Arc::new(Served::default());
	served.set("button.json", BUTTON);
	served.set("utils.json", UTILS);
	let dir = tempfile::tempdir()?;
	std::fs::create_dir(dir.path().join("src"))?;
	let resolver = Resolver::new(dir.path()).fetch(served.clone()).cache(None);
	let engine = Engine::builder().user_config(UserConfig::default()).open(dir.path())?;
	let source = format!("registry:{BUTTON_URL}");
	let first = resolver.resolve(&source, None)?;

	let plan = engine.plan(&PieceRequest {
		pack: &first,
		piece: "button",
		inputs: &Values::new(),
		variant: None,
		target: None,
		granted: None,
		force: false,
	})?;
	assert_eq!(plan.summary(), "add registry-button/button (with utils)");
	engine.apply(plan)?;
	let button = dir.path().join("src/components/ui/button.tsx");
	std::fs::write(&button, format!("// mine\n{}", std::fs::read_to_string(&button)?))?;
	served.set("button.json", &BUTTON.replace("label = 1", "label = 2"));
	let second = resolver.resolve(&source, None)?;
	engine.apply(engine.plan_update(
		&UpdateRequest {
			pack: &second,
			granted: None,
			on_conflict: OnConflict::Refuse,
			force: false,
			migrate: true,
		},
		std::slice::from_ref(&first),
	)?)?;

	assert_eq!(
		std::fs::read_to_string(&button)?,
		"// mine\nimport { cn } from \"@/lib/utils\"\nexport const label = 2\n"
	);
	assert_eq!(engine.lockfile()?.packs[0].source.as_deref(), Some(source.as_str()));
	assert!(dir.path().join("src/lib/utils.ts").exists());
	Ok(())
}

#[test]
fn pinned_should_load_what_the_lockfile_records_and_report_the_rest() -> TestResult {
	let served = Arc::new(Served::default());
	served.set("button.json", BUTTON);
	served.set("utils.json", UTILS);
	let dir = tempfile::tempdir()?;
	let resolver = Resolver::for_project(dir.path(), &ProjectConfig::default(), &UserConfig::default())?
		.fetch(served)
		.cache(Some(Cache::new(dir.path().join("cache"))));
	let pack = resolver.resolve(&format!("registry:{BUTTON_URL}"), None)?;
	let record = |source: &str, digest| PackRecord {
		name: pack.manifest().name.clone(),
		version: pack.manifest().version.clone(),
		source: Some(source.to_owned()),
		digest,
		signer: None,
	};
	let lock = Lockfile {
		packs: vec![
			record(&format!("registry:{BUTTON_URL}"), pack.digest()),
			record("official:ui", pack.digest()),
			record(
				"registry:https://registry.example/r/gone.json",
				Digest::of(b"never cached"),
			),
		],
		..Lockfile::default()
	};

	let (loaded, failed) = resolver.pinned(&lock);

	assert_eq!(loaded.len(), 1);
	assert_eq!(loaded[0].digest(), pack.digest());
	assert_eq!(failed.len(), 1);
	assert_eq!(failed[0].0, "registry:https://registry.example/r/gone.json");
	Ok(())
}
