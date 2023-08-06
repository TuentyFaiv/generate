use std::time::Duration;
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::utils::command;
use crate::cli::structs::Answers;
use crate::cli::enums::ArchType;
use crate::cli::actions::{create_url, rm_git, cp_envs, install};

use super::CLIGlobalTemplates;

pub fn generate(CLIGlobalTemplates {
  answers,
  config,
  error
}: &CLIGlobalTemplates) -> Result<()> {
  let Answers { name, path, tool, tool_type, arch, .. } = answers;
  
  match tool_type {
    Some(tool_type) => match config.find_repository(&tool, &tool_type) {
      Some(repo) => {
        let repository = config.get_repository(&repo.name.unwrap_or(String::new()));
        let url = create_url(&repository);

        let is_library = arch == &ArchType::Library;
        let arch = arch.to_string();

        let pb = ProgressBar::new(1000);
        pb.set_style(ProgressStyle::with_template("\n{spinner:.green} {msg}").unwrap());
        pb.enable_steady_tick(Duration::from_millis(50));
        pb.set_message("Creating...");

        command(
          "git",
          ["clone", url.as_ref(), &path].to_vec(),
          None,
          Some(format!("Failed to generate {} with {arch}", tool.to_string()).as_str())
        );

        if is_library {
          match repo.library {
            Some(library) => {
              command("git", ["switch", &library].to_vec(), Some(&path), Some("Failed to switch to library"));
              Ok(())
            },
            None => Err(anyhow!("Branch for library not exist"))
          }?;
        }

        pb.set_message("Initialize git...");
        rm_git(&path);

        pb.set_message("Initial commit...");
        command("git", ["init", "-b", "main"].to_vec(), Some(&path), Some("Failed to restart git"));

        command("git", ["add", "."].to_vec(), Some(&path), Some("Failed to staging files"));

        let type_msg = if is_library { "library" } else { "project" };
        let commit = format!("🎉 FEAT: Starting {type_msg} {}", name.original);
        command("git", ["commit", "-m", commit.as_str(), "--no-gpg-sign"].to_vec(), Some(&path), Some("Failed to commit"));

        command("git", ["remote", "add", "template", url.as_ref()].to_vec(), Some(&path), Some("Failed to add remote repository"));

        if !is_library {
          pb.set_message("Creating env files...");
          cp_envs(&path);
        }
        pb.set_message("Instaling dependencies...");
        install(&path);
        pb.finish_and_clear();
        Ok(())
      },
      None => Err(anyhow!(error.clone()))
    },
    None => Err(anyhow!(error.clone()))
  }
}