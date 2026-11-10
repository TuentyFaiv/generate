//! Stdio JSON-RPC 2.0 API for the engine: one message per line, plans applied by id.
//! One engine serves the whole session, so the project config is read once.

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value as Json, json};
use tfverse_core::plan::{OnConflict, Plan, Selection, Validated};
use tfverse_core::{
	CaptureRequest, Engine, EngineError, Event, EventSink, Pack, PieceRequest, RelPath, UpdateRequest, UserConfig, Values,
};
use tfverse_packs::{Library, PackSpec, Resolver, SourceError};

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const ENGINE_ERROR: i64 = -32000;

/// Serves requests from `input` until it ends, writing responses and events to `output`.
pub fn serve<W: Write + Send + 'static>(root: &Path, input: impl BufRead, output: W) -> io::Result<W> {
	let user = UserConfig::load().map_err(io::Error::other)?;
	serve_with(root, user, input, output)
}

/// Like [`serve`], with an explicit user config; returns `output` when `input` ends.
pub fn serve_with<W: Write + Send + 'static>(
	root: &Path,
	user: UserConfig,
	input: impl BufRead,
	output: W,
) -> io::Result<W> {
	let output = Output(Arc::new(Mutex::new(output)));
	let mut session = Session::new(root, user, output.clone()).map_err(io::Error::other)?;
	for line in input.lines() {
		let line = line?;
		if line.trim().is_empty() {
			continue;
		}
		if let Some(response) = session.handle(&line) {
			output.write(&response)?;
		}
	}
	drop(session);
	Arc::try_unwrap(output.0)
		.map(|output| output.into_inner().unwrap_or_else(PoisonError::into_inner))
		.map_err(|_| io::Error::other("output is still in use"))
}

/// The shared output stream, written one message per line.
struct Output<W>(Arc<Mutex<W>>);

impl<W> Clone for Output<W> {
	fn clone(&self) -> Self {
		Self(Arc::clone(&self.0))
	}
}

impl<W: Write> Output<W> {
	fn write(&self, message: &Json) -> io::Result<()> {
		let mut output = self.0.lock().unwrap_or_else(PoisonError::into_inner);
		serde_json::to_writer(&mut *output, message)?;
		output.write_all(b"\n")?;
		output.flush()
	}
}

impl<W: Write + Send> EventSink for Output<W> {
	fn emit(&self, event: &Event<'_>) {
		let params = match event {
			Event::Wrote(path) => json!({ "kind": "wrote", "path": path }),
			Event::Removed(path) => json!({ "kind": "removed", "path": path }),
			Event::Applied { transaction, changes } => {
				json!({ "kind": "applied", "transaction": transaction, "changes": changes })
			}
			Event::RolledBack(transaction) => json!({ "kind": "rolled_back", "transaction": transaction }),
			Event::Undone(transaction) => json!({ "kind": "undone", "transaction": transaction }),
			_ => return,
		};
		let _ = self.write(&json!({ "jsonrpc": "2.0", "method": "event", "params": params }));
	}
}

/// A failed call: a JSON-RPC error code, a message and optional details.
#[derive(Debug)]
pub struct RpcError {
	/// JSON-RPC error code.
	pub code: i64,
	/// What went wrong.
	pub message: String,
	/// Details, such as the problems of a rejected plan.
	pub data: Option<Json>,
}

/// The JSON-RPC methods as direct calls, for servers such as MCP that speak another protocol.
pub struct Calls {
	session: Session<io::Sink>,
}

impl Calls {
	/// A session for the project at `root`; engine events are dropped.
	pub fn open(root: &Path, user: UserConfig) -> Result<Self, String> {
		let output = Output(Arc::new(Mutex::new(io::sink())));
		Ok(Self {
			session: Session::new(root, user, output).map_err(|error| error.to_string())?,
		})
	}

	/// Calls `method` with `params`, exactly as a JSON-RPC request would.
	pub fn call(&mut self, method: &str, params: Json) -> Result<Json, RpcError> {
		self.session.call(method, params)
	}
}

impl RpcError {
	fn new(code: i64, message: impl Into<String>) -> Self {
		Self {
			code,
			message: message.into(),
			data: None,
		}
	}
}

impl From<SourceError> for RpcError {
	fn from(error: SourceError) -> Self {
		Self::new(ENGINE_ERROR, error.to_string())
	}
}

impl From<EngineError> for RpcError {
	fn from(error: EngineError) -> Self {
		let data = match &error {
			EngineError::Rejected(rejected) => serde_json::to_value(rejected).ok(),
			_ => None,
		};
		Self {
			code: ENGINE_ERROR,
			message: error.to_string(),
			data,
		}
	}
}

struct Session<W> {
	engine: LazyEngine<W>,
	packs: Library,
	plans: BTreeMap<u64, Plan<Validated>>,
	next_plan: u64,
}

/// Opens the engine on first use, and again after a failed open.
struct LazyEngine<W> {
	root: PathBuf,
	user: UserConfig,
	output: Output<W>,
	engine: Option<Engine>,
}

impl<W: Write + Send + 'static> LazyEngine<W> {
	fn get(&mut self) -> Result<&Engine, RpcError> {
		let engine = match self.engine.take() {
			Some(engine) => engine,
			None => Engine::builder()
				.user_config(self.user.clone())
				.sink(self.output.clone())
				.open(&self.root)?,
		};
		Ok(self.engine.insert(engine))
	}
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoadPack {
	path: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AddPack {
	source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PieceRef {
	pack: String,
	piece: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatePlan {
	pack: String,
	piece: String,
	#[serde(default)]
	inputs: Values,
	#[serde(default)]
	variant: Option<String>,
	#[serde(default)]
	target: Option<RelPath>,
	#[serde(default)]
	force: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdatePlan {
	pack: String,
	#[serde(default)]
	source: Option<String>,
	#[serde(default)]
	on_conflict: OnConflict,
	#[serde(default)]
	force: bool,
	#[serde(default = "yes")]
	migrate: bool,
}

fn yes() -> bool {
	true
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
	pack_dir: RelPath,
	pack: String,
	piece: String,
	files: Vec<RelPath>,
	#[serde(default)]
	example: Option<String>,
	#[serde(default)]
	force: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectPlan {
	plan: u64,
	keep: Selection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanRef {
	plan: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Why {
	path: RelPath,
	line: u32,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Undo {
	#[serde(default)]
	force: bool,
}

impl<W: Write + Send + 'static> Session<W> {
	fn new(root: &Path, user: UserConfig, output: Output<W>) -> Result<Self, tfverse_core::pack::PackError> {
		Ok(Self {
			engine: LazyEngine {
				root: root.to_path_buf(),
				user,
				output,
				engine: None,
			},
			packs: Library::official()?,
			plans: BTreeMap::new(),
			next_plan: 1,
		})
	}

	fn handle(&mut self, line: &str) -> Option<Json> {
		let message: Json = match serde_json::from_str(line) {
			Ok(message) => message,
			Err(error) => {
				return Some(error_response(
					&Json::Null,
					&RpcError::new(PARSE_ERROR, error.to_string()),
				));
			}
		};
		let id = message.get("id").cloned();
		let respond_to = id.clone().unwrap_or(Json::Null);
		let valid = message.get("jsonrpc") == Some(&json!("2.0")) && message.get("method").is_some_and(Json::is_string);
		if !valid {
			return Some(error_response(
				&respond_to,
				&RpcError::new(INVALID_REQUEST, "expected a JSON-RPC 2.0 request object"),
			));
		}
		let method = message["method"].as_str().unwrap_or_default();
		let params = message.get("params").cloned().unwrap_or(Json::Null);

		let result = self.call(method, params);
		let id = id?;
		Some(match result {
			Ok(value) => json!({ "jsonrpc": "2.0", "id": id, "result": value }),
			Err(error) => error_response(&id, &error),
		})
	}

	fn call(&mut self, method: &str, params: Json) -> Result<Json, RpcError> {
		match method {
			"project.detect" => to_json(&self.engine.get()?.project()?),
			"pack.add" => {
				let AddPack { source } = parse(params)?;
				self.add_pack(&source)
			}
			"pack.load" => {
				let LoadPack { path } = parse(params)?;
				self.add_pack(&PackSpec::Path(path).to_string())
			}
			"pack.list" => Ok(Json::Array(self.packs.iter().map(pack_summary).collect())),
			"piece.describe" => {
				let PieceRef { pack, piece } = parse(params)?;
				let found = find_pack(&self.packs, &pack)?
					.piece(&piece)
					.ok_or_else(|| RpcError::new(INVALID_PARAMS, format!("no piece `{piece}`")))?;
				to_json(found.manifest())
			}
			"plan.create" => self.create_plan(&parse(params)?),
			"plan.update" => self.update_plan(&parse(params)?),
			"plan.adopt" => self.adopt_plan(&parse(params)?),
			"pack.capture" => {
				let Capture {
					pack_dir,
					pack,
					piece,
					files,
					example,
					force,
				} = parse(params)?;
				let plan = self.engine.get()?.plan_capture(&CaptureRequest {
					pack_dir: &pack_dir,
					pack: &pack,
					piece: &piece,
					files: &files,
					example: example.as_deref(),
					force,
				})?;
				self.keep_plan(plan)
			}
			"plan.apply" => {
				let PlanRef { plan } = parse(params)?;
				let plan = self
					.plans
					.remove(&plan)
					.ok_or_else(|| RpcError::new(INVALID_PARAMS, format!("no pending plan {plan}")))?;
				to_json(&self.engine.get()?.apply(plan)?)
			}
			"plan.hunks" => {
				let PlanRef { plan } = parse(params)?;
				let plan = self.pending(plan)?;
				Ok(Json::Array(
					plan
						.changes()
						.iter()
						.map(|change| json!({ "path": change.path, "action": change.action, "hunks": change.hunks() }))
						.collect(),
				))
			}
			"plan.select" => {
				let SelectPlan { plan, keep } = parse(params)?;
				let pending = self.pending_mut(plan)?;
				pending
					.select(&keep)
					.map_err(|error| RpcError::new(INVALID_PARAMS, error.to_string()))?;
				Ok(json!({ "plan": plan, "report": to_json(&pending.report())? }))
			}
			"plan.discard" => {
				let PlanRef { plan } = parse(params)?;
				Ok(json!(self.plans.remove(&plan).is_some()))
			}
			"status" => to_json(&self.engine.get()?.status()?),
			"why" => {
				let Why { path, line } = parse(params)?;
				to_json(&self.engine.get()?.why(&path, line, &self.packs)?)
			}
			"history.list" => to_json(&self.engine.get()?.history()?),
			"history.undo" => {
				let Undo { force } = if params.is_null() {
					Undo::default()
				} else {
					parse(params)?
				};
				to_json(&self.engine.get()?.undo(force)?)
			}
			_ => Err(RpcError::new(METHOD_NOT_FOUND, format!("unknown method `{method}`"))),
		}
	}

	/// Loads the pack at `source`, pinned to the hash the lockfile recorded for that source.
	fn add_pack(&mut self, source: &str) -> Result<Json, RpcError> {
		let pack = self.resolve(source, true)?;
		let summary = pack_summary(&pack);
		self.packs.add(pack);
		Ok(summary)
	}

	/// Loads `source`; when `pinned`, to the hash the lockfile recorded for that source.
	fn resolve(&mut self, source: &str, pinned: bool) -> Result<Pack, RpcError> {
		let spec: PackSpec = source
			.parse()
			.map_err(|error: tfverse_packs::SpecError| RpcError::new(INVALID_PARAMS, error.to_string()))?;
		let canonical = spec.to_string();
		let root = self.engine.root.clone();
		let user = self.engine.user.clone();
		let engine = self.engine.get()?;
		let digest = if pinned {
			engine
				.lockfile()?
				.packs
				.iter()
				.find(|pack| pack.source.as_deref() == Some(canonical.as_str()))
				.map(|pack| pack.digest)
		} else {
			None
		};
		Ok(Resolver::for_project(root, engine.config(), &user)?.resolve(source, digest)?)
	}

	fn adopt_plan(&mut self, params: &CreatePlan) -> Result<Json, RpcError> {
		let pack = find_pack(&self.packs, &params.pack)?;
		let plan = self.engine.get()?.plan_adopt(&PieceRequest {
			pack,
			piece: &params.piece,
			inputs: &params.inputs,
			variant: params.variant.as_deref(),
			target: params.target.as_ref(),
			granted: None,
			force: params.force,
		})?;
		self.keep_plan(plan)
	}

	fn create_plan(&mut self, params: &CreatePlan) -> Result<Json, RpcError> {
		let pack = find_pack(&self.packs, &params.pack)?;
		let plan = self.engine.get()?.plan(&PieceRequest {
			pack,
			piece: &params.piece,
			inputs: &params.inputs,
			variant: params.variant.as_deref(),
			target: params.target.as_ref(),
			granted: None,
			force: params.force,
		})?;
		self.keep_plan(plan)
	}

	/// Plans updating the project to `source` fetched fresh, or else to the newest loaded version of a pack.
	fn update_plan(&mut self, params: &UpdatePlan) -> Result<Json, RpcError> {
		self.load_pinned(&params.pack)?;
		let fresh = params
			.source
			.as_deref()
			.map(|source| self.resolve(source, false))
			.transpose()?;
		if let Some(fresh) = &fresh
			&& fresh.manifest().name.as_str() != params.pack
		{
			return Err(RpcError::new(
				INVALID_PARAMS,
				format!(
					"`{}` is pack `{}`, not `{}`",
					fresh.source().unwrap_or_default(),
					fresh.manifest().name,
					params.pack
				),
			));
		}
		let pack = match &fresh {
			Some(fresh) => fresh,
			None => find_pack(&self.packs, &params.pack)?,
		};
		let plan = self.engine.get()?.plan_update(
			&UpdateRequest {
				pack,
				granted: None,
				on_conflict: params.on_conflict,
				force: params.force,
				migrate: params.migrate,
			},
			&self.packs,
		)?;
		self.keep_plan(plan)
	}

	/// Loads the version the lockfile pins from its source, so originals can be rebuilt; best effort,
	/// since most originals are already cached in the project.
	fn load_pinned(&mut self, name: &str) -> Result<(), RpcError> {
		let root = self.engine.root.clone();
		let user = self.engine.user.clone();
		let engine = self.engine.get()?;
		let lock = engine.lockfile()?;
		let Some(record) = lock.packs.iter().find(|pack| pack.name.as_str() == name) else {
			return Ok(());
		};
		let loaded = self
			.packs
			.versions(name)
			.any(|pack| pack.manifest().version == record.version);
		if let (false, Some(source)) = (loaded, &record.source)
			&& let Ok(pack) = Resolver::for_project(root, engine.config(), &user)?.resolve(source, Some(record.digest))
			&& pack.manifest().version == record.version
		{
			self.packs.add(pack);
		}
		Ok(())
	}

	fn pending(&self, id: u64) -> Result<&Plan<Validated>, RpcError> {
		self
			.plans
			.get(&id)
			.ok_or_else(|| RpcError::new(INVALID_PARAMS, format!("no pending plan {id}")))
	}

	fn pending_mut(&mut self, id: u64) -> Result<&mut Plan<Validated>, RpcError> {
		self
			.plans
			.get_mut(&id)
			.ok_or_else(|| RpcError::new(INVALID_PARAMS, format!("no pending plan {id}")))
	}

	fn keep_plan(&mut self, plan: Plan<Validated>) -> Result<Json, RpcError> {
		let report = to_json(&plan.report())?;
		let id = self.next_plan;
		self.next_plan += 1;
		self.plans.insert(id, plan);
		Ok(json!({ "plan": id, "report": report }))
	}
}

fn find_pack<'a>(packs: &'a Library, name: &str) -> Result<&'a Pack, RpcError> {
	packs
		.get(name)
		.ok_or_else(|| RpcError::new(INVALID_PARAMS, format!("pack `{name}` is not loaded")))
}

fn pack_summary(pack: &Pack) -> Json {
	let manifest = pack.manifest();
	json!({
		"name": manifest.name,
		"version": manifest.version.to_string(),
		"source": pack.source(),
		"digest": pack.digest(),
		"description": manifest.description,
		"permissions": manifest.permissions,
		"pieces": pack.pieces().map(|piece| json!({
			"name": piece.manifest().name,
			"kind": piece.manifest().kind,
			"description": piece.manifest().description,
		})).collect::<Vec<_>>(),
	})
}

fn parse<T: DeserializeOwned>(params: Json) -> Result<T, RpcError> {
	serde_json::from_value(params).map_err(|error| RpcError::new(INVALID_PARAMS, error.to_string()))
}

fn to_json(value: &impl serde::Serialize) -> Result<Json, RpcError> {
	serde_json::to_value(value).map_err(|error| RpcError::new(ENGINE_ERROR, error.to_string()))
}

fn error_response(id: &Json, error: &RpcError) -> Json {
	let mut body = json!({ "code": error.code, "message": error.message });
	if let Some(data) = &error.data {
		body["data"] = data.clone();
	}
	json!({ "jsonrpc": "2.0", "id": id, "error": body })
}
