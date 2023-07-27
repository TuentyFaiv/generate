use crate::cli::command;

pub fn create_url(repository: &str) -> String {
  if cfg!(target_os = "windows") {
    let repo_win = repository.replace("git@", "https://").replace("github.com:", "github.com/");
    return repo_win;
  }

  repository.to_string()
}

pub fn rm_git(path: &str) {
  if cfg!(target_os = "windows") {
    command("cmd", ["/C", "rmdir", "/s", "/q", ".git"].to_vec(), Some(path), Some("Failed to reset git"));
  } else {
    command("rm", ["-rf", ".git"].to_vec(), Some(path), Some("Failed to reset git"));
  }
}

pub fn cp_envs(path: &str) {
  if cfg!(target_os = "windows") {
    command("cmd", ["/C", "copy", ".env.example", ".env.development"].to_vec(), Some(path), Some("Failed to create development env file"));
    command("cmd", ["/C", "copy", ".env.example", ".env.production"].to_vec(), Some(path), Some("Failed to create production env file"));
  } else {
    command("cp", [".env.example", ".env.development"].to_vec(), Some(path), Some("Failed to create development env file"));
    command("cp", [".env.example", ".env.production"].to_vec(), Some(path), Some("Failed to create production env file"));
  }
}

pub fn install(path: &str) {
  if cfg!(target_os = "windows") {
    command("cmd", ["/C", "pnpm", "install"].to_vec(), Some(path), Some("Failed to install dependencies"));
  } else {
    command("pnpm", ["install"].to_vec(), Some(path), Some("Failed to install dependencies"));
  }
}

