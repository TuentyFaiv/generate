use std::time::Duration;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::{command, done, msg};

fn in_windons(path: &str) {
  command("cmd", ["/C", "rmdir", "/s", "/q", ".git"].to_vec(), Some(path), Some("Failed to reset git"));
}

fn in_unix(path: &str) {
  command("rm", ["-rf", ".git"].to_vec(), Some(path), Some("Failed to reset git"));
}

pub fn make(
  template: &Vec<&str>,
  name: &str,
  path: &str,
  tool: &str,
  arch: &str
) {
  let pb = ProgressBar::new(1000);
  pb.set_style(ProgressStyle::with_template("\n{spinner:.green} {msg}").unwrap());
  pb.enable_steady_tick(Duration::from_millis(50));
  pb.set_message("Creating...");

  let repository = *template.get(1).unwrap();
  let commit = format!("🎉 FEAT: Starting project {name}");
  let url = if cfg!(target_os = "windows") {
    format!("https://github.com/Platimex/{repository}.git")
  } else {
    format!("git@github.com:Platimex/{repository}.git")
  };

  command(
    "git",
    ["clone", url.as_str(), path].to_vec(),
    None,
    Some(format!("Failed to generate {tool} with {arch}").as_str())
  );

  // if create_library {
  // 	command("git", ["switch", "library"].to_vec(), Some(&path), Some("Failed to switch to library"));
  // }

  pb.set_message("Reseting git...");
  if cfg!(target_os = "windows") {
    in_windons(path);
  } else {
    in_unix(path);
  }

  pb.set_message("Adding initial commint...");
  command("git", ["init", "-b", "main"].to_vec(), Some(path), Some("Failed to restart git"));

  command("git", ["add", "."].to_vec(), Some(path), Some("Failed to staging files"));

  command("git", ["commit", "-m", commit.as_str(), "-m", "\"\"", "--no-gpg-sign"].to_vec(), Some(path), Some("Failed to commit"));

  command("git", ["remote", "add", "template", url.as_str()].to_vec(), Some(path), Some("Failed to add remote repository"));

  pb.set_message("Creating env files...");
  command("cp", [".env.example", ".env.development"].to_vec(), Some(path), Some("Failed to create development env file"));
  command("cp", [".env.example", ".env.production"].to_vec(), Some(path), Some("Failed to create production env file"));

  pb.set_message("Instaling dependencies...");
  command("pnpm", ["install"].to_vec(), Some(path), Some("Failed to install dependencies"));
  pb.finish_and_clear();

  done();
  msg(&format!("{}", style(format!("Move to {path} and start a new universe")).cyan()));
}