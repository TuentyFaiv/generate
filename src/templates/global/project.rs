use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::{command};
use crate::cli::actions::{create_url, rm_git, cp_envs, install};

pub fn generate(
  repository: &str,
  name: &str,
  path: &str,
  tool: &str,
  arch: &str,
  is_library: bool
) {
  let pb = ProgressBar::new(1000);
  pb.set_style(ProgressStyle::with_template("\n{spinner:.green} {msg}").unwrap());
  pb.enable_steady_tick(Duration::from_millis(50));
  pb.set_message("Creating...");

  let type_msg = if is_library { "library" } else { "project" };
  let commit = format!("🎉 FEAT: Starting {type_msg} {name}");
  let url = create_url(repository);

  command(
    "git",
    ["clone", url.as_str(), path].to_vec(),
    None,
    Some(format!("Failed to generate {tool} with {arch}").as_str())
  );

  if is_library {
  	command("git", ["switch", "library"].to_vec(), Some(&path), Some("Failed to switch to library"));
  }

  pb.set_message("Reseting git...");
  rm_git(path);

  pb.set_message("Adding initial commint...");
  command("git", ["init", "-b", "main"].to_vec(), Some(path), Some("Failed to restart git"));

  command("git", ["add", "."].to_vec(), Some(path), Some("Failed to staging files"));

  command("git", ["commit", "-m", commit.as_str(), "-m", "\"\"", "--no-gpg-sign"].to_vec(), Some(path), Some("Failed to commit"));

  command("git", ["remote", "add", "template", url.as_str()].to_vec(), Some(path), Some("Failed to add remote repository"));

  pb.set_message("Creating env files...");
  cp_envs(path);

  pb.set_message("Instaling dependencies...");
  install(path);
  pb.finish_and_clear();
}