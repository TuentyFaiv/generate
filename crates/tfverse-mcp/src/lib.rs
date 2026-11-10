//! MCP server for `tfverse`: the plan → review → apply flow as tools for AI agents, over stdio.
//!
//! Every tool is one engine call; plans are only ever written by `apply_plan`, so an agent can always
//! show the user what will change first.

use std::io::{self, BufRead, Write};
use std::path::Path;

use serde_json::{Value as Json, json};
use tfverse_core::UserConfig;
use tfverse_rpc::Calls;

const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const PARSE_ERROR: i64 = -32700;

const INSTRUCTIONS: &str = "tfverse generates code from packs. Find a piece with list_packs and describe_piece, \
plan it with plan_add (or plan_update, plan_adopt, plan_capture), show the user the plan's changes, then call \
apply_plan with the plan id. Nothing is written before apply_plan; undo reverts the last apply.";

/// One tool: its name, the engine method behind it, what it does, its arguments and whether it writes.
struct Tool {
	name: &'static str,
	method: &'static str,
	description: &'static str,
	properties: fn() -> Json,
	required: &'static [&'static str],
	writes: bool,
}

fn nothing() -> Json {
	json!({})
}

fn piece_arguments() -> Json {
	json!({
		"pack": { "type": "string", "description": "Pack name, e.g. `ui`." },
		"piece": { "type": "string", "description": "Piece name, e.g. `atom`." },
		"inputs": { "type": "object", "description": "Input values by name; see describe_piece.", "additionalProperties": true },
		"variant": { "type": "string", "description": "Variant instead of the detected one." },
		"target": { "type": "string", "description": "Output folder instead of the piece's." },
		"force": { "type": "boolean", "description": "Replace existing files that differ." }
	})
}

const TOOLS: &[Tool] = &[
	Tool {
		name: "detect_project",
		method: "project.detect",
		description: "Detect the project's stack: language, framework, styling, package manager and more.",
		properties: nothing,
		required: &[],
		writes: false,
	},
	Tool {
		name: "list_packs",
		method: "pack.list",
		description: "List the loaded packs and their pieces.",
		properties: nothing,
		required: &[],
		writes: false,
	},
	Tool {
		name: "add_pack",
		method: "pack.add",
		description: "Load a pack from a source: `./folder`, `gh:owner/repo@tag`, an HTTPS `.tar.gz`, `git:` over SSH, `registry:` items or `@namespace/pack`.",
		properties: || json!({ "source": { "type": "string" } }),
		required: &["source"],
		writes: false,
	},
	Tool {
		name: "describe_piece",
		method: "piece.describe",
		description: "Describe a piece: its inputs, variants, files and patches.",
		properties: || json!({ "pack": { "type": "string" }, "piece": { "type": "string" } }),
		required: &["pack", "piece"],
		writes: false,
	},
	Tool {
		name: "plan_add",
		method: "plan.create",
		description: "Plan generating a piece (and the pieces it requires). Returns a plan id and its changes; nothing is written.",
		properties: piece_arguments,
		required: &["pack", "piece"],
		writes: false,
	},
	Tool {
		name: "plan_adopt",
		method: "plan.adopt",
		description: "Plan taking over existing code as a piece, so later updates merge into it. Existing files are not changed.",
		properties: piece_arguments,
		required: &["pack", "piece"],
		writes: false,
	},
	Tool {
		name: "plan_update",
		method: "plan.update",
		description: "Plan updating everything generated from a pack to its newest loaded version, or to `source`, merging the user's edits.",
		properties: || {
			json!({
				"pack": { "type": "string" },
				"source": { "type": "string", "description": "Load the new version from here first." },
				"on_conflict": { "type": "string", "enum": ["refuse", "markers", "ours", "theirs"] },
				"force": { "type": "boolean" },
				"migrate": { "type": "boolean", "description": "Run the pack's migrations (default true)." }
			})
		},
		required: &["pack"],
		writes: false,
	},
	Tool {
		name: "plan_capture",
		method: "pack.capture",
		description: "Plan turning project files into a new piece of a local pack.",
		properties: || {
			json!({
				"pack_dir": { "type": "string", "description": "e.g. `packs/local`" },
				"pack": { "type": "string" },
				"piece": { "type": "string" },
				"files": { "type": "array", "items": { "type": "string" } },
				"example": { "type": "string", "description": "How the name is spelled in the files, e.g. `UserCard`." },
				"force": { "type": "boolean" }
			})
		},
		required: &["pack_dir", "pack", "piece", "files"],
		writes: false,
	},
	Tool {
		name: "plan_hunks",
		method: "plan.hunks",
		description: "List each change of a pending plan as hunks that can be kept or skipped.",
		properties: || json!({ "plan": { "type": "integer" } }),
		required: &["plan"],
		writes: false,
	},
	Tool {
		name: "select_hunks",
		method: "plan.select",
		description: "Keep only some changes of a pending plan: per path, `all`, `none` or a list of hunk indexes.",
		properties: || {
			json!({
				"plan": { "type": "integer" },
				"keep": { "type": "object", "additionalProperties": true }
			})
		},
		required: &["plan", "keep"],
		writes: false,
	},
	Tool {
		name: "apply_plan",
		method: "plan.apply",
		description: "Apply a pending plan as one transaction. Show the user the plan first.",
		properties: || json!({ "plan": { "type": "integer" } }),
		required: &["plan"],
		writes: true,
	},
	Tool {
		name: "discard_plan",
		method: "plan.discard",
		description: "Drop a pending plan.",
		properties: || json!({ "plan": { "type": "integer" } }),
		required: &["plan"],
		writes: false,
	},
	Tool {
		name: "status",
		method: "status",
		description: "Show every generated file and whether it is clean, modified or missing.",
		properties: nothing,
		required: &[],
		writes: false,
	},
	Tool {
		name: "why",
		method: "why",
		description: "Explain where a line of a generated file comes from: the pack, or the user.",
		properties: || json!({ "path": { "type": "string" }, "line": { "type": "integer", "minimum": 1 } }),
		required: &["path", "line"],
		writes: false,
	},
	Tool {
		name: "history",
		method: "history.list",
		description: "List applied transactions that can be undone.",
		properties: nothing,
		required: &[],
		writes: false,
	},
	Tool {
		name: "undo",
		method: "history.undo",
		description: "Undo the last transaction; refuses when its files changed since, unless `force`.",
		properties: || json!({ "force": { "type": "boolean" } }),
		required: &[],
		writes: true,
	},
];

/// Serves MCP over `input` and `output`, one JSON message per line, until `input` ends.
pub fn serve(root: &Path, user: UserConfig, input: impl BufRead, mut output: impl Write) -> io::Result<()> {
	let mut calls = Calls::open(root, user).map_err(io::Error::other)?;
	for line in input.lines() {
		let line = line?;
		if line.trim().is_empty() {
			continue;
		}
		if let Some(response) = handle(&mut calls, &line) {
			serde_json::to_writer(&mut output, &response)?;
			output.write_all(b"\n")?;
			output.flush()?;
		}
	}
	Ok(())
}

fn handle(calls: &mut Calls, line: &str) -> Option<Json> {
	let message: Json = match serde_json::from_str(line) {
		Ok(message) => message,
		Err(error) => return Some(error_response(&Json::Null, PARSE_ERROR, &error.to_string())),
	};
	let id = message.get("id").cloned()?;
	let method = message.get("method").and_then(Json::as_str).unwrap_or_default();
	let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
	let result = match method {
		"initialize" => Ok(initialize(&params)),
		"ping" => Ok(json!({})),
		"tools/list" => Ok(json!({ "tools": TOOLS.iter().map(describe).collect::<Vec<_>>() })),
		"tools/call" => call_tool(calls, &params),
		_ => Err((METHOD_NOT_FOUND, format!("unknown method `{method}`"))),
	};
	Some(match result {
		Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
		Err((code, message)) => error_response(&id, code, &message),
	})
}

fn initialize(params: &Json) -> Json {
	let requested = params.get("protocolVersion").and_then(Json::as_str);
	let version = requested
		.filter(|version| PROTOCOL_VERSIONS.contains(version))
		.unwrap_or(PROTOCOL_VERSIONS[0]);
	json!({
		"protocolVersion": version,
		"capabilities": { "tools": { "listChanged": false } },
		"serverInfo": { "name": tfverse_core::brand::NAME, "version": env!("CARGO_PKG_VERSION") },
		"instructions": INSTRUCTIONS,
	})
}

fn describe(tool: &Tool) -> Json {
	json!({
		"name": tool.name,
		"description": tool.description,
		"inputSchema": {
			"type": "object",
			"properties": (tool.properties)(),
			"required": tool.required,
			"additionalProperties": false,
		},
		"annotations": {
			"readOnlyHint": !tool.writes,
			"destructiveHint": tool.writes,
			"idempotentHint": false,
			"openWorldHint": tool.name == "add_pack" || tool.name == "plan_update",
		},
	})
}

fn call_tool(calls: &mut Calls, params: &Json) -> Result<Json, (i64, String)> {
	let name = params.get("name").and_then(Json::as_str).unwrap_or_default();
	let tool = TOOLS
		.iter()
		.find(|tool| tool.name == name)
		.ok_or_else(|| (INVALID_PARAMS, format!("unknown tool `{name}`")))?;
	let arguments = match params.get("arguments") {
		None | Some(Json::Null) => json!({}),
		Some(arguments) => arguments.clone(),
	};
	let arguments = if arguments.as_object().is_some_and(serde_json::Map::is_empty) && tool.required.is_empty() {
		Json::Null
	} else {
		arguments
	};
	Ok(match calls.call(tool.method, arguments) {
		Ok(value) => {
			let mut result = json!({
				"content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }],
				"isError": false,
			});
			if value.is_object() {
				result["structuredContent"] = value;
			}
			result
		}
		Err(error) => {
			let mut text = error.message;
			if let Some(data) = error.data {
				text.push('\n');
				text.push_str(&serde_json::to_string_pretty(&data).unwrap_or_default());
			}
			json!({ "content": [{ "type": "text", "text": text }], "isError": true })
		}
	})
}

fn error_response(id: &Json, code: i64, message: &str) -> Json {
	json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
