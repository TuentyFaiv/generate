//! Drives whole MCP sessions over in-memory stdio.

use std::error::Error;
use std::path::Path;

use serde_json::{Value, json};
use tfverse_core::UserConfig;

type TestResult = Result<(), Box<dyn Error>>;

fn session(root: &Path, requests: &[Value]) -> Result<Vec<Value>, Box<dyn Error>> {
	let input = requests.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
	let mut output = Vec::new();
	tfverse_mcp::serve(root, UserConfig::default(), input.as_bytes(), &mut output)?;
	Ok(
		String::from_utf8(output)?
			.lines()
			.map(serde_json::from_str)
			.collect::<Result<_, _>>()?,
	)
}

fn call(id: u64, tool: &str, arguments: &Value) -> Value {
	json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": tool, "arguments": arguments } })
}

#[test]
fn initialize_should_agree_on_a_protocol_version_and_list_tools() -> TestResult {
	let dir = tempfile::tempdir()?;
	let replies = session(
		dir.path(),
		&[
			json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } } }),
			json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
			json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
		],
	)?;

	assert_eq!(replies.len(), 2);
	assert_eq!(replies[0]["result"]["protocolVersion"], "2025-06-18");
	let tools: Vec<&str> = replies[1]["result"]["tools"]
		.as_array()
		.ok_or("tools should be a list")?
		.iter()
		.filter_map(|tool| tool["name"].as_str())
		.collect();
	assert!(
		tools.contains(&"plan_add") && tools.contains(&"apply_plan"),
		"{tools:?}"
	);
	Ok(())
}

#[test]
fn an_agent_should_plan_then_apply_an_official_piece() -> TestResult {
	let dir = tempfile::tempdir()?;
	std::fs::write(
		dir.path().join("package.json"),
		r#"{"dependencies": {"react": "19"}, "devDependencies": {"typescript": "6"}}"#,
	)?;
	let replies = session(
		dir.path(),
		&[
			call(
				1,
				"plan_add",
				&json!({ "pack": "ui", "piece": "atom", "inputs": { "name": "Badge" } }),
			),
			call(2, "apply_plan", &json!({ "plan": 1 })),
			call(3, "status", &json!({})),
		],
	)?;

	assert_eq!(replies[0]["result"]["isError"], false, "{:?}", replies[0]);
	assert_eq!(replies[0]["result"]["structuredContent"]["plan"], 1);
	assert_eq!(replies[1]["result"]["isError"], false, "{:?}", replies[1]);
	assert!(dir.path().join("src/ui/atoms/badge").is_dir());
	assert!(
		replies[2]["result"]["content"][0]["text"]
			.as_str()
			.is_some_and(|text| text.contains("clean"))
	);
	Ok(())
}

#[test]
fn engine_errors_should_be_tool_errors_and_unknown_tools_protocol_errors() -> TestResult {
	let dir = tempfile::tempdir()?;
	let replies = session(
		dir.path(),
		&[
			call(1, "plan_add", &json!({ "pack": "ui", "piece": "nope" })),
			call(2, "nope", &json!({})),
		],
	)?;

	assert_eq!(replies[0]["result"]["isError"], true);
	assert_eq!(replies[1]["error"]["code"], -32602);
	Ok(())
}
