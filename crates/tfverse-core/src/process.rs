//! Running a user-configured program with input on stdin, a time limit and bounded output.

use std::io::{Read as _, Write as _};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(20);
const MAX_OUTPUT: u64 = 16 * 1024 * 1024;

/// What a finished program wrote.
pub(crate) struct Output {
	/// How it exited.
	pub status: ExitStatus,
	/// Standard output, up to 16 MB.
	pub stdout: Vec<u8>,
	/// Standard error, up to 16 MB.
	pub stderr: Vec<u8>,
}

/// Why a program could not run to the end.
#[derive(Debug, thiserror::Error)]
pub(crate) enum RunError {
	/// The program could not be started.
	#[error("could not start `{program}`: {source}")]
	Start {
		/// The program.
		program: String,
		/// The cause.
		source: std::io::Error,
	},
	/// It ran longer than allowed and was stopped.
	#[error("stopped after {0} seconds")]
	TimedOut(u64),
	/// Waiting for it failed.
	#[error("{0}")]
	Wait(std::io::Error),
}

impl RunError {
	/// Whether the program is not installed.
	pub(crate) fn not_found(&self) -> bool {
		matches!(self, Self::Start { source, .. } if source.kind() == std::io::ErrorKind::NotFound)
	}
}

/// Runs `program` with `args`, writing `input` to its stdin, and stops it after `timeout`.
pub(crate) fn run(program: &str, args: &[String], input: Vec<u8>, timeout: Duration) -> Result<Output, RunError> {
	let mut child = Command::new(program)
		.args(args)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.map_err(|source| RunError::Start {
			program: program.to_owned(),
			source,
		})?;
	let stdin = child.stdin.take();
	let writer = thread::spawn(move || stdin.map(|mut stdin| stdin.write_all(&input)));
	let read = |pipe: Option<Box<dyn std::io::Read + Send>>| {
		thread::spawn(move || {
			let mut bytes = Vec::new();
			if let Some(pipe) = pipe {
				let _ = pipe.take(MAX_OUTPUT).read_to_end(&mut bytes);
			}
			bytes
		})
	};
	let stdout = read(
		child
			.stdout
			.take()
			.map(|pipe| Box::new(pipe) as Box<dyn std::io::Read + Send>),
	);
	let stderr = read(
		child
			.stderr
			.take()
			.map(|pipe| Box::new(pipe) as Box<dyn std::io::Read + Send>),
	);

	let started = Instant::now();
	let status = loop {
		match child.try_wait().map_err(RunError::Wait)? {
			Some(status) => break status,
			None if started.elapsed() > timeout => {
				let _ = child.kill();
				let _ = child.wait();
				return Err(RunError::TimedOut(timeout.as_secs()));
			}
			None => thread::sleep(POLL),
		}
	};
	let _ = writer.join();
	Ok(Output {
		status,
		stdout: stdout.join().unwrap_or_default(),
		stderr: stderr.join().unwrap_or_default(),
	})
}
