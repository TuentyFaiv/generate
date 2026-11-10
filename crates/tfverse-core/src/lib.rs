//! Core engine for `tfverse`: packs, rendering, plan → apply, history and provenance.
//!
//! Layers, lowest first: [`model`] → template → [`patch`] → [`pack`] → [`workspace`] → [`history`] →
//! [`plan`] → [`engine`]. Each layer only uses the ones below it.

pub mod brand;
pub mod engine;
pub mod event;
pub mod history;
pub mod migrate;
pub mod model;
pub mod pack;
pub mod patch;
pub mod plan;
mod process;
pub mod schema;
pub mod template;
pub mod workspace;

pub use engine::{CaptureRequest, Engine, EngineBuilder, EngineError, PackProvider, PieceRequest, UpdateRequest};
pub use event::{Event, EventSink, NoopSink};
pub use model::{RelPath, Value, Values};
pub use pack::Pack;
pub use workspace::{DiskFs, Fs, MemoryFs, UserConfig};
