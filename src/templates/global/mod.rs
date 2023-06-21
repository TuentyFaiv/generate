mod statics;

use std::io::{Write, Read, BufReader};
use std::fs::File;
use std::time::Duration;
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};

use crate::statics::NOT_IMPLEMENTED;
use crate::utils::{change_case};
use crate::config::CLIConfig;
use crate::cli::{enums::ArchType, structs::Answers};

pub struct CLIGlobalTemplates {
  answers: Answers,
  config: CLIConfig,
  error: String,
}

impl CLIGlobalTemplates {
  pub fn new(config: CLIConfig, answers: Answers) -> Self {
    let error = format!("{} Repository not found", NOT_IMPLEMENTED);
    Self { config, answers, error }
  }
  pub fn generate_project(&self) -> Result<()> {
    // For project generation
    use crate::cli::{command};
    use crate::cli::actions::{create_url, rm_git, cp_envs, install};

    let Answers { name, path, tool, tool_type, arch, .. } = self.answers.clone();
    
    match tool_type {
      Some(tool_type) => match self.config.find_repository(&tool, &tool_type) {
        Some(repo) => {
          let repository = self.config.get_repository(&repo.name.unwrap_or(String::new()));
          let url = create_url(&repository);

          let is_library = arch == ArchType::Library;
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
          let commit = format!("🎉 FEAT: Starting {type_msg} {name}");
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
        None => Err(anyhow!(self.error.clone()))
      },
      None => Err(anyhow!(self.error.clone()))
    }
  }
  pub fn generate_schema(&self) -> Result<()> {
    // For schema generation
    use statics::schema::{PROPTYPES, SCHEMA, SCHEMA_TS};
    use statics::schema::{NEW_IMPORT, TYPE_EXPORT};

    let path = "";
    let path_proptypes = "";
    let name = "";
    let name_dash = "";
    let namespace = "";
    let is_ts = true;
    let name_camel = change_case(name, Some("camel"));
  
    let mut proptypes = PROPTYPES.to_string();
    let mut schema = SCHEMA.to_string();
    let mut schema_import = NEW_IMPORT.to_string();
    let mut type_export = TYPE_EXPORT.to_string();
    let mut ext = ".js".to_string();

    if is_ts {
      schema = SCHEMA_TS.to_string();
      ext = ".ts".to_string();
    }

    proptypes = proptypes.replace("NAMESPACE", namespace);
    
    schema = schema.replace("NAME_DASH", name_dash);
    schema = schema.replace("NAME", name);

    schema_import = schema_import.replace("NAME_CAMEL", &name_camel);
    type_export = type_export.replace("NAME", name);

    let index_path = format!("{path}/index.ts");
    let schema_path = format!("{path}/{name_camel}{ext}");
    let proptypes_path = format!("{path_proptypes}/{namespace}{ext}");

    let mut schema_file = File::create(schema_path)?;
    schema_file.write_all(schema.as_bytes())?;

    match File::open(&index_path) {
      Ok(index_file) => {
        let mut buf_reader = BufReader::new(&index_file);
        let mut index_content = String::new();
        buf_reader.read_to_string(&mut index_content)?;

        let mut new_index = File::create(&index_path)?;
        let updated_index = [index_content.as_str(), schema_import.as_str()].concat();
        new_index.write_all(updated_index.as_bytes())?;
      },
      Err(_) => {
        let mut index_file = File::create(&index_path)?;

        index_file.write_all(schema_import.as_bytes())?;
      }
    };

    if is_ts {
      match File::open(&proptypes_path) {
        Ok(proptypes_file) => {
          let mut buf_reader = BufReader::new(&proptypes_file);
          let mut proptypes_content = String::new();
          buf_reader.read_to_string(&mut proptypes_content)?;
    
          let mut new_proptypes = File::create(&proptypes_path)?;
          
          proptypes_content = proptypes_content.replace("// NEXT_TYPE", &type_export);
    
          new_proptypes.write_all(proptypes_content.as_bytes())?;
        },
        Err(_) => {
          let mut proptypes_file = File::create(&proptypes_path)?;
    
          proptypes = proptypes.replace("// NEXT_TYPE", &type_export);
    
          proptypes_file.write_all(proptypes.as_bytes())?;
        }
      };
    }

    Ok(())
  }
  pub fn generate_service(&self) -> Result<()> {
    // For service generation
    use statics::service::{PROPTYPES, SERVICE, SERVICE_TS};
    use statics::service::{SERVICE_IMPORT, TYPE_EXPORT, INSTANCES};

    let path = "";
    let path_proptypes = "";
    let path_instances = "";
    let name = "";
    let namespace = "";
    let is_ts = true;

    let instances = INSTANCES.to_string();
    let name_camel = change_case(name, Some("camel"));
    
    let mut proptypes = PROPTYPES.to_string();
    let mut service = SERVICE.to_string();
    let mut service_import = SERVICE_IMPORT.to_string();
    let mut type_export = TYPE_EXPORT.to_string();
    let mut ext = ".js".to_string();

    if is_ts {
      service = SERVICE_TS.to_string();
      ext = ".ts".to_string();
    }

    type_export = type_export.replace("NAME", name);
    
    service = service.replace("NAME_CAMEL", &name_camel);
    service = service.replace("NAMESPACE", namespace);
    service = service.replace("NAME", name);

    service_import = service_import.replace("NAME_CAMEL", &name_camel);

    let index_path = format!("{path}/index.ts");
    let instances_path = format!("{path_instances}/instances.ts");
    let service_path = format!("{path}/{name_camel}{ext}");
    let proptypes_path = format!("{path_proptypes}/{namespace}{ext}");

    let mut service_file = File::create(service_path)?;
    service_file.write_all(service.as_bytes())?;

    match File::open(&index_path) {
      Ok(index_file) => {
        let mut buf_reader = BufReader::new(&index_file);
        let mut index_content = String::new();
        buf_reader.read_to_string(&mut index_content)?;

        let mut new_index = File::create(&index_path)?;
        let updated_index = [index_content.as_str(), service_import.as_str()].concat();
        new_index.write_all(updated_index.as_bytes())?;
      },
      Err(_) => {
        let mut index_file = File::create(&index_path)?;

        index_file.write_all(service_import.as_bytes())?;
      }
    };

    match File::open(&instances_path) {
      Ok(_) => {},
      Err(_) => {
        let mut instances_file = File::create(&instances_path)?;

        instances_file.write_all(instances.as_bytes())?;
      }
    }

    if is_ts {
      match File::open(&proptypes_path) {
        Ok(proptypes_file) => {
          let mut buf_reader = BufReader::new(&proptypes_file);
          let mut proptypes_content = String::new();
          buf_reader.read_to_string(&mut proptypes_content)?;
    
          let mut new_proptypes = File::create(&proptypes_path)?;
          
          proptypes_content = proptypes_content.replace("// NEXT_TYPE", &type_export);
    
          new_proptypes.write_all(proptypes_content.as_bytes())?;
        },
        Err(_) => {
          let mut proptypes_file = File::create(&proptypes_path)?;

          proptypes = proptypes.replace("// NEXT_TYPE", &format!("{type_export}\n"));
    
          proptypes_file.write_all(proptypes.as_bytes())?;
        }
      };
    }

    Ok(())
  }
}