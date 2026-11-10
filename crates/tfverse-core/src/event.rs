//! Progress events, so the CLI, MCP server and GUI can follow what the engine does.

use crate::model::RelPath;

/// Something the engine did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event<'a> {
	/// A file was written.
	Wrote(&'a RelPath),
	/// A file was removed.
	Removed(&'a RelPath),
	/// A transaction finished.
	Applied {
		/// Transaction id.
		transaction: u64,
		/// Files changed.
		changes: usize,
	},
	/// A failed transaction was rolled back.
	RolledBack(u64),
	/// A transaction was undone.
	Undone(u64),
}

/// Receives engine events.
pub trait EventSink: Send + Sync {
	/// Called for every event, in order.
	fn emit(&self, event: &Event<'_>);
}

/// Ignores every event.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopSink;

impl EventSink for NoopSink {
	fn emit(&self, _: &Event<'_>) {}
}

impl<F: Fn(&Event<'_>) + Send + Sync> EventSink for F {
	fn emit(&self, event: &Event<'_>) {
		self(event);
	}
}
