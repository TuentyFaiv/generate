//! Drives whole JSON-RPC sessions over in-memory stdio.

use std::error::Error;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tfverse_core::UserConfig;

type TestResult = Result<(), Box<dyn Error>>;

fn fixture_pack() -> String {
	Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("../tfverse-core/tests/fixtures/sample-pack")
		.display()
		.to_string()
}

fn project() -> Result<tempfile::TempDir, Box<dyn Error>> {
	let dir = tempfile::tempdir()?;
	fs::write(dir.path().join("package.json"), r#"{"dependencies": {"react": "1"}}"#)?;
	Ok(dir)
}

fn session(root: &Path, requests: &[Value]) -> Result<Vec<Value>, Box<dyn Error>> {
	let input = requests.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
	let output = tfverse_rpc::serve_with(root, UserConfig::default(), input.as_bytes(), Vec::new())?;
	Ok(
		String::from_utf8(output)?
			.lines()
			.map(serde_json::from_str)
			.collect::<Result<_, _>>()?,
	)
}

fn responses(messages: &[Value]) -> Vec<&Value> {
	messages.iter().filter(|message| message.get("id").is_some()).collect()
}

fn request(id: u64, method: &str, params: Value) -> Value {
	let mut request = json!({ "jsonrpc": "2.0", "id": id, "method": method });
	request["params"] = params;
	request
}

#[test]
fn plan_then_apply_should_write_files_and_emit_events() -> TestResult {
	let dir = project()?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.load", json!({ "path": fixture_pack() })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "UserCard" } }),
			),
			request(3, "plan.apply", json!({ "plan": 1 })),
		],
	)?;

	let wrote = messages
		.iter()
		.any(|message| message["method"] == "event" && message["params"]["kind"] == "wrote");

	assert!(wrote && dir.path().join("src/components/user-card.tsx").exists());
	Ok(())
}

#[test]
fn plan_create_should_return_the_report() -> TestResult {
	let dir = project()?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.load", json!({ "path": fixture_pack() })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "UserCard" } }),
			),
		],
	)?;

	assert_eq!(
		responses(&messages)[1]["result"]["report"]["changes"][0]["path"],
		"src/components/user-card.tsx"
	);
	Ok(())
}

#[test]
fn a_plan_should_apply_only_once() -> TestResult {
	let dir = project()?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.load", json!({ "path": fixture_pack() })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "A" } }),
			),
			request(3, "plan.apply", json!({ "plan": 1 })),
			request(4, "plan.apply", json!({ "plan": 1 })),
		],
	)?;

	assert_eq!(responses(&messages)[3]["error"]["code"], -32602);
	Ok(())
}

#[test]
fn rejected_plan_should_include_problems() -> TestResult {
	let dir = project()?;
	fs::create_dir_all(dir.path().join("src/components"))?;
	fs::write(dir.path().join("src/components/A.tsx"), "mine")?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.load", json!({ "path": fixture_pack() })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "A" } }),
			),
		],
	)?;

	assert_eq!(
		responses(&messages)[1]["error"]["data"]["problems"][0]["kind"],
		"conflict"
	);
	Ok(())
}

#[test]
fn undo_should_revert_the_last_apply() -> TestResult {
	let dir = project()?;
	session(
		dir.path(),
		&[
			request(1, "pack.load", json!({ "path": fixture_pack() })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "A" } }),
			),
			request(3, "plan.apply", json!({ "plan": 1 })),
			request(4, "history.undo", Value::Null),
		],
	)?;

	assert!(!dir.path().join("src/components/A.tsx").exists());
	Ok(())
}

#[test]
fn unknown_method_should_fail_with_method_not_found() -> TestResult {
	let dir = project()?;
	let messages = session(dir.path(), &[request(1, "rm.rf", Value::Null)])?;

	assert_eq!(messages[0]["error"]["code"], -32601);
	Ok(())
}

#[test]
fn invalid_json_should_fail_with_parse_error() -> TestResult {
	let dir = project()?;
	let output = tfverse_rpc::serve_with(dir.path(), UserConfig::default(), "{not json\n".as_bytes(), Vec::new())?;

	let message: Value = serde_json::from_slice(&output)?;

	assert_eq!(message["error"]["code"], -32700);
	Ok(())
}

#[test]
fn notifications_should_get_no_response() -> TestResult {
	let dir = project()?;
	let messages = session(dir.path(), &[json!({ "jsonrpc": "2.0", "method": "status" })])?;

	assert_eq!(messages.len(), 0);
	Ok(())
}

#[test]
fn unknown_params_should_be_rejected() -> TestResult {
	let dir = project()?;
	let messages = session(
		dir.path(),
		&[request(
			1,
			"pack.load",
			json!({ "path": fixture_pack(), "trust": true }),
		)],
	)?;

	assert_eq!(messages[0]["error"]["code"], -32602);
	Ok(())
}

#[test]
fn project_detect_should_report_the_stack() -> TestResult {
	let dir = project()?;
	let messages = session(dir.path(), &[request(1, "project.detect", Value::Null)])?;

	assert_eq!(messages[0]["result"]["stack"]["framework"], json!(["react"]));
	Ok(())
}

#[test]
fn official_packs_should_be_available_without_loading() -> TestResult {
	let dir = project()?;
	let messages = session(dir.path(), &[request(1, "pack.list", Value::Null)])?;

	let names: Vec<&str> = responses(&messages)[0]["result"]
		.as_array()
		.ok_or("no list")?
		.iter()
		.filter_map(|pack| pack["name"].as_str())
		.collect();
	assert_eq!(names, ["backend", "ui"]);
	Ok(())
}

#[test]
fn official_resource_plan_should_add_error_and_list_dependencies() -> TestResult {
	let dir = tempfile::tempdir()?;
	fs::write(
		dir.path().join("package.json"),
		r#"{"dependencies": {"hono": "4"}, "devDependencies": {"typescript": "5"}}"#,
	)?;
	let messages = session(
		dir.path(),
		&[request(
			1,
			"plan.create",
			json!({ "pack": "backend", "piece": "resource", "inputs": { "name": "Invoice", "plural": "Invoices" } }),
		)],
	)?;

	let report = &responses(&messages)[0]["result"]["report"];
	assert_eq!(report["summary"], "add backend/resource (with error)");
	let dependencies: Vec<&str> = report["dependencies"]
		.as_array()
		.ok_or("no dependencies")?
		.iter()
		.filter_map(|dependency| dependency["name"].as_str())
		.collect();
	assert_eq!(dependencies, ["zod", "vitest"]);
	Ok(())
}

#[test]
fn pack_add_should_load_a_folder_and_record_its_source() -> TestResult {
	let dir = project()?;
	fs::create_dir_all(dir.path().join("packs"))?;
	copy_dir(Path::new(&fixture_pack()), &dir.path().join("packs/sample"))?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.add", json!({ "source": "./packs/sample" })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "UserCard" } }),
			),
			request(3, "plan.apply", json!({ "plan": 1 })),
		],
	)?;

	assert_eq!(responses(&messages)[0]["result"]["source"], "path:./packs/sample");
	let lock = fs::read_to_string(dir.path().join("tfverse.lock"))?;
	assert!(lock.contains("source = \"path:./packs/sample\""), "{lock}");
	Ok(())
}

#[test]
fn plan_update_should_move_generated_files_to_the_new_version() -> TestResult {
	let dir = project()?;
	copy_dir(Path::new(&fixture_pack()), &dir.path().join("packs/v1"))?;
	copy_dir(Path::new(&fixture_pack()), &dir.path().join("packs/v2"))?;
	let manifest = dir.path().join("packs/v2/pack.toml");
	fs::write(
		&manifest,
		fs::read_to_string(&manifest)?.replace("version = \"1.0.0\"", "version = \"1.1.0\""),
	)?;
	let component = dir.path().join("packs/v2/pieces/component/files/react/my-example.tsx");
	fs::write(&component, format!("{}// updated\n", fs::read_to_string(&component)?))?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.add", json!({ "source": "./packs/v1" })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "UserCard" } }),
			),
			request(3, "plan.apply", json!({ "plan": 1 })),
			request(4, "pack.add", json!({ "source": "./packs/v2" })),
			request(5, "plan.update", json!({ "pack": "sample", "on_conflict": "markers" })),
			request(6, "plan.apply", json!({ "plan": 2 })),
		],
	)?;

	let replies = responses(&messages);
	assert_eq!(
		replies[4]["result"]["report"]["summary"], "update sample 1.0.0 → 1.1.0",
		"{replies:?}"
	);
	assert!(fs::read_to_string(dir.path().join("src/components/user-card.tsx"))?.ends_with("// updated\n"));
	let lock = fs::read_to_string(dir.path().join("tfverse.lock"))?;
	assert!(lock.contains("source = \"path:./packs/v2\""), "{lock}");
	Ok(())
}

#[test]
fn plan_select_should_skip_files_before_apply() -> TestResult {
	let dir = project()?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.load", json!({ "path": fixture_pack() })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "UserCard", "styled": true } }),
			),
			request(3, "plan.hunks", json!({ "plan": 1 })),
			request(
				4,
				"plan.select",
				json!({ "plan": 1, "keep": { "src/components/user-card.module.css": "none" } }),
			),
			request(5, "plan.select", json!({ "plan": 1, "keep": { "src/nope.ts": "all" } })),
			request(6, "plan.apply", json!({ "plan": 1 })),
		],
	)?;

	let replies = responses(&messages);
	let hunks = replies[2]["result"].as_array().ok_or("hunks should be a list")?;
	assert!(
		hunks.iter().all(|change| change["hunks"][0]["header"].is_string()),
		"{hunks:?}"
	);
	assert_eq!(
		replies[3]["result"]["report"]["changes"].as_array().map(Vec::len),
		Some(1)
	);
	assert_eq!(replies[4]["error"]["code"], -32602);
	assert!(dir.path().join("src/components/user-card.tsx").exists());
	assert!(!dir.path().join("src/components/user-card.module.css").exists());
	Ok(())
}

#[test]
fn plan_update_should_load_the_new_version_from_a_source() -> TestResult {
	let dir = project()?;
	copy_dir(Path::new(&fixture_pack()), &dir.path().join("packs/v1"))?;
	copy_dir(Path::new(&fixture_pack()), &dir.path().join("packs/v2"))?;
	let manifest = dir.path().join("packs/v2/pack.toml");
	fs::write(
		&manifest,
		fs::read_to_string(&manifest)?.replace("version = \"1.0.0\"", "version = \"1.1.0\""),
	)?;
	let messages = session(
		dir.path(),
		&[
			request(1, "pack.add", json!({ "source": "./packs/v1" })),
			request(
				2,
				"plan.create",
				json!({ "pack": "sample", "piece": "component", "inputs": { "name": "UserCard" } }),
			),
			request(3, "plan.apply", json!({ "plan": 1 })),
			request(4, "plan.update", json!({ "pack": "sample", "source": "./packs/v2" })),
			request(5, "plan.update", json!({ "pack": "other", "source": "./packs/v2" })),
		],
	)?;

	let replies = responses(&messages);
	assert_eq!(
		replies[3]["result"]["report"]["summary"], "update sample 1.0.0 → 1.1.0",
		"{replies:?}"
	);
	assert_eq!(replies[4]["error"]["code"], -32602);
	Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
	fs::create_dir_all(to)?;
	for entry in fs::read_dir(from)? {
		let entry = entry?;
		let target = to.join(entry.file_name());
		if entry.file_type()?.is_dir() {
			copy_dir(&entry.path(), &target)?;
		} else {
			fs::copy(entry.path(), target)?;
		}
	}
	Ok(())
}
