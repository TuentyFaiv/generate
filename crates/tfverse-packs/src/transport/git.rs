//! `git:` sources over SSH, fetched with the user's own git so their SSH keys and agent apply.

use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use super::fetch::{FetchError, MAX_DOWNLOAD};

/// How long one git command may run before it is stopped.
pub const GIT_TIMEOUT: Duration = Duration::from_secs(120);

/// SSH options when the user has not chosen an SSH command: never prompt, give up on dead hosts.
const SSH_COMMAND: &str = "ssh -o BatchMode=yes -o ConnectTimeout=30";

/// Fetches one commit of a repository as a tar archive; a trait so tests can serve repositories.
pub trait Git: Send + Sync + fmt::Debug {
	/// The files of `url` at `reference`, as a tar archive with a single top folder.
	fn archive(&self, url: &str, reference: &str) -> Result<Vec<u8>, FetchError>;
}

/// Refuses every `git:` source, for offline use and builds without the `remote` feature.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoGit;

impl Git for NoGit {
	fn archive(&self, url: &str, _reference: &str) -> Result<Vec<u8>, FetchError> {
		Err(FetchError {
			url: url.to_owned(),
			reason: if cfg!(feature = "remote") {
				"network sources are turned off".to_owned()
			} else {
				"this build of tfverse cannot fetch git sources".to_owned()
			},
		})
	}
}

/// Runs the `git` on `PATH` over SSH only, never prompting, and stops commands that take too long.
#[derive(Debug)]
pub struct SystemGit {
	timeout: Duration,
	allow_local: bool,
}

impl Default for SystemGit {
	fn default() -> Self {
		Self {
			timeout: GIT_TIMEOUT,
			allow_local: false,
		}
	}
}

static NEXT: AtomicU64 = AtomicU64::new(0);

impl SystemGit {
	/// Stops each git command after `timeout`.
	#[must_use]
	pub fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}

	/// Also allows local repositories; for tests.
	#[cfg(test)]
	pub(crate) fn allowing_local() -> Self {
		Self {
			allow_local: true,
			..Self::default()
		}
	}

	fn command(&self, dir: &Path, ssh: Option<&str>) -> Command {
		let mut command = Command::new("git");
		command
			.current_dir(dir)
			.env("GIT_TERMINAL_PROMPT", "0")
			.env("GIT_ASKPASS", "")
			.env("SSH_ASKPASS_REQUIRE", "never")
			.args(["-c", "protocol.allow=never"])
			.args(["-c", "protocol.ssh.allow=always"])
			.stdin(Stdio::null());
		if let Some(ssh) = ssh {
			command.env("GIT_SSH_COMMAND", ssh);
		}
		if self.allow_local {
			command.args(["-c", "protocol.file.allow=always"]);
		}
		command
	}

	/// [`SSH_COMMAND`], unless the user chose an SSH command of their own.
	fn ssh_command(&self, dir: &Path) -> Option<&'static str> {
		if std::env::var_os("GIT_SSH_COMMAND").is_some() || std::env::var_os("GIT_SSH").is_some() {
			return None;
		}
		let configured = self
			.command(dir, None)
			.args(["config", "--get", "core.sshCommand"])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.status()
			.is_ok_and(|status| status.success());
		(!configured).then_some(SSH_COMMAND)
	}
}

impl Git for SystemGit {
	fn archive(&self, url: &str, reference: &str) -> Result<Vec<u8>, FetchError> {
		let error = |reason: String| FetchError {
			url: url.to_owned(),
			reason,
		};
		if url.starts_with('-') || reference.starts_with('-') {
			return Err(error("must not start with `-`".to_owned()));
		}
		let dir = TempDir::new().map_err(|source| error(format!("cannot create a work folder: {source}")))?;
		let run = |args: &[&str], ssh: Option<&str>| -> Result<Vec<u8>, FetchError> {
			let child = self
				.command(&dir.0, ssh)
				.args(args)
				.stdout(Stdio::piped())
				.stderr(Stdio::piped())
				.spawn()
				.map_err(|source| error(format!("cannot run git: {source}")))?;
			let finished = wait(child, self.timeout).map_err(|reason| error(format!("git {}: {reason}", args[0])))?;
			if finished.status.success() {
				Ok(finished.stdout)
			} else {
				Err(error(format!(
					"git {} failed: {}",
					args[0],
					String::from_utf8_lossy(&finished.stderr).trim()
				)))
			}
		};
		run(&["init", "--quiet"], None)?;
		let ssh = self.ssh_command(&dir.0);
		run(&["fetch", "--quiet", "--depth", "1", "--no-tags", url, reference], ssh)?;
		run(&["archive", "--format=tar", "--prefix=root/", "FETCH_HEAD"], None)
	}
}

/// What a finished command wrote.
struct Finished {
	status: ExitStatus,
	stdout: Vec<u8>,
	stderr: Vec<u8>,
}

/// Waits for `child`, keeping at most [`MAX_DOWNLOAD`] bytes of output; kills it after `timeout`.
fn wait(mut child: Child, timeout: Duration) -> Result<Finished, String> {
	let read = |pipe: Option<Box<dyn Read + Send>>| {
		thread::spawn(move || {
			let mut bytes = Vec::new();
			if let Some(pipe) = pipe {
				let _ = pipe.take(MAX_DOWNLOAD + 1).read_to_end(&mut bytes);
			}
			bytes
		})
	};
	let stdout = read(child.stdout.take().map(|pipe| Box::new(pipe) as Box<dyn Read + Send>));
	let stderr = read(child.stderr.take().map(|pipe| Box::new(pipe) as Box<dyn Read + Send>));
	let deadline = Instant::now() + timeout;
	let status = loop {
		match child.try_wait() {
			Ok(Some(status)) => break status,
			Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
			Ok(None) => {
				let _ = child.kill();
				let _ = child.wait();
				return Err(format!("stopped after {} seconds", timeout.as_secs()));
			}
			Err(error) => return Err(error.to_string()),
		}
	};
	let stdout = stdout.join().unwrap_or_default();
	if stdout.len() as u64 > MAX_DOWNLOAD {
		return Err(format!("larger than {MAX_DOWNLOAD} bytes"));
	}
	Ok(Finished {
		status,
		stdout,
		stderr: stderr.join().unwrap_or_default(),
	})
}

/// A work folder removed when dropped.
struct TempDir(PathBuf);

impl TempDir {
	fn new() -> std::io::Result<Self> {
		let id = NEXT.fetch_add(1, Ordering::Relaxed);
		let path = std::env::temp_dir().join(format!("tfverse-git-{}-{id}", std::process::id()));
		std::fs::create_dir_all(&path)?;
		Ok(Self(path))
	}
}

impl Drop for TempDir {
	fn drop(&mut self) {
		let _ = std::fs::remove_dir_all(&self.0);
	}
}

#[cfg(test)]
pub(crate) mod tests {
	use std::process::Command;

	use super::*;
	use crate::format::unpack_tar;

	/// A local repository with `files` committed and tagged `v1.0.0`.
	pub(crate) fn repository(files: &[(&str, &str)]) -> tempfile::TempDir {
		let dir = tempfile::tempdir().unwrap();
		let git = |args: &[&str]| {
			let status = Command::new("git")
				.current_dir(dir.path())
				.args([
					"-c",
					"user.name=t",
					"-c",
					"user.email=t@t",
					"-c",
					"commit.gpgsign=false",
				])
				.args(["-c", "tag.gpgsign=false", "-c", "init.defaultBranch=main"])
				.args(args)
				.stdout(Stdio::null())
				.stderr(Stdio::null())
				.status()
				.unwrap();
			assert!(status.success(), "git {args:?}");
		};
		git(&["init", "--quiet"]);
		for (path, text) in files {
			let file = dir.path().join(path);
			std::fs::create_dir_all(file.parent().unwrap()).unwrap();
			std::fs::write(file, text).unwrap();
		}
		git(&["add", "."]);
		git(&["commit", "--quiet", "-m", "pack"]);
		git(&["tag", "v1.0.0"]);
		dir
	}

	#[test]
	fn archive_should_fetch_a_tag_without_a_checkout() {
		let repo = repository(&[("pack.toml", "x"), ("pieces/a.txt", "y")]);

		let tar = SystemGit::allowing_local()
			.archive(&repo.path().display().to_string(), "v1.0.0")
			.unwrap();

		let files = unpack_tar(&tar, true).unwrap();
		assert_eq!(files.keys().collect::<Vec<_>>(), ["pack.toml", "pieces/a.txt"]);
	}

	#[test]
	fn archive_should_refuse_local_repositories_by_default() {
		let repo = repository(&[("pack.toml", "x")]);

		let error = SystemGit::default()
			.archive(&repo.path().display().to_string(), "v1.0.0")
			.unwrap_err();

		assert!(error.reason.contains("transport 'file' not allowed"), "{error}");
	}

	#[cfg(unix)]
	#[test]
	fn wait_should_stop_a_command_that_runs_too_long() {
		let child = Command::new("sleep")
			.arg("5")
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.unwrap();
		let started = Instant::now();

		let result = wait(child, Duration::from_millis(100));

		assert!(matches!(result, Err(reason) if reason.contains("stopped")));
		assert!(started.elapsed() < Duration::from_secs(2));
	}

	#[test]
	fn archive_should_refuse_https_repositories() {
		let error = SystemGit::default()
			.archive("https://example.com/repo.git", "v1")
			.unwrap_err();

		assert!(error.reason.contains("transport 'https' not allowed"), "{error}");
	}

	#[test]
	fn no_git_should_refuse_every_source() {
		assert!(NoGit.archive("git@example.com:a/b.git", "v1").is_err());
	}

	#[test]
	fn archive_should_refuse_option_like_arguments() {
		assert!(SystemGit::default().archive("--upload-pack=touch x", "v1").is_err());
	}
}
