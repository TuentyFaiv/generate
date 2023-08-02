use std::io::{Write, Read, BufReader};
use std::fs::File;
use std::time::Duration;
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};

use crate::statics::NOT_IMPLEMENTED;
use crate::statics::global;
use crate::utils::{change_case, read_path};
use crate::cli::enums::{Tool, ArchType};
use crate::cli::structs::Answers;
use crate::create::structs::{ComponentCreation, PageCreation, LayoutCreation};
use crate::config::CLIConfig;

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
  pub fn generate_component(&self, templates: ComponentCreation, tool: &Tool) -> Result<()> {
    let name = self.answers.name.as_str();

    let template_path = match tool {
      Tool::React => templates.react_path(),
      Tool::Svelte => templates.svelte_path(),
      Tool::Vanilla => templates.vanilla_path(),
    };

    let mut styles = read_path(
      &template_path,
      templates.styles.template,
      templates.styles.default
    );
    let mut responsive = read_path(
      &template_path,
      templates.responsive.template,
      templates.responsive.default
    );
    let mut component = read_path(
      &template_path,
      templates.component.template,
      templates.component.default
    );
    if let Some(template_script) = templates.script {
      let script = read_path(
        &template_path,
        template_script.template,
        template_script.default,
      );
      component = component.replace("SCRIPT", &script);
    }
    let component_import = templates.import;

    component = component.replace("NAME_LOWER", &name.to_lowercase());
    component = component.replace("NAME", name);
    styles = styles.replace("NAME_LOWER", &name.to_lowercase());
    styles = styles.replace("NAME", name);
    responsive = responsive.replace("NAME_LOWER", &name.to_lowercase());
    responsive = responsive.replace("NAME", name);

    let mut component_file = File::create(templates.exports.component)?;
    let mut styles_file = File::create(templates.exports.styles)?;
    let mut responsive_file = File::create(templates.exports.responsive)?;

    component_file.write_all(component.as_bytes())?;
    styles_file.write_all(styles.as_bytes())?;
    responsive_file.write_all(responsive.as_bytes())?;

    let index_path = templates.exports.barrel;
    match File::open(&index_path) {
      Ok(index) => {
        let mut buf_reader = BufReader::new(&index);
        let mut index_content = String::new();
        buf_reader.read_to_string(&mut index_content)?;

        if index_content.contains("export {};") {
          index_content = index_content.replace("export {};", "");
        }

        let mut new_index = File::create(&index_path)?;
        let updated_index = [index_content.as_str(), component_import.as_str()].concat();

        new_index.write_all(updated_index.as_bytes())?;
      },
      Err(_) => {
        let mut index = File::create(&index_path)?;

        index.write_all(component_import.as_bytes())?;
      }
    }

    if let Some(template_props) = templates.proptypes {
      let mut proptypes = read_path(
        &template_path,
        template_props.template,
        template_props.default,
      );
      proptypes = proptypes.replace("NAME_LOWER", &name.to_lowercase());
      proptypes = proptypes.replace("NAME", name);
      if let Some(export_props) = templates.exports.proptypes {
        let mut proptypes_file = File::create(export_props)?;
        proptypes_file.write_all(proptypes.as_bytes())?;
      }
    }

    Ok(())
  }
  pub fn generate_project(&self) -> Result<()> {
    use crate::cli::utils::command;
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
  pub fn generate_page(&self, templates: PageCreation, tool: &Tool) -> Result<()> {
    let name = self.answers.name.as_str();
    let nampe_capital = change_case(name, None);

    let template_path = match tool {
      Tool::React => templates.react_path(),
      Tool::Svelte => templates.svelte_path(),
      Tool::Vanilla => templates.vanilla_path(),
    };

    let mut config = read_path(
      &template_path,
      templates.aliases.config.template,
      templates.aliases.config.default
    );
    let mut aliases = read_path(
      &template_path,
      templates.aliases.config_aliases.template,
      templates.aliases.config_aliases.default
    );
    let mut styles = read_path(
      &template_path,
      templates.styles.template,
      templates.styles.default
    );
    let mut responsive = read_path(
      &template_path,
      templates.responsive.template,
      templates.responsive.default
    );
    let mut page = read_path(
      &template_path,
      templates.page.template,
      templates.page.default
    );
    if let Some(template_script) = templates.script {
      let script = read_path(
        &template_path,
        template_script.template,
        template_script.default,
      );
      page = page.replace("SCRIPT", &script);
    }

    let styles_import = templates.imports.styles;

    page = page.replace("NAME_CAPITAL", &nampe_capital);
    page = page.replace("NAME_LOWER", &name.to_lowercase());
    page = page.replace("NAME", name);
    styles = styles.replace("NAME_CAPITAL", &nampe_capital);
    styles = styles.replace("NAME_LOWER", &name.to_lowercase());
    styles = styles.replace("NAME", name);
    responsive = responsive.replace("NAME_CAPITAL", &nampe_capital);
    responsive = responsive.replace("NAME_LOWER", &name.to_lowercase());
    responsive = responsive.replace("NAME", name);
    aliases = aliases.replace("NAME_CAPITAL", &nampe_capital);
    aliases = aliases.replace("NAME_LOWER", &name.to_lowercase());
    aliases = aliases.replace("NAME", name);

    let mut page_file = File::create(templates.exports.page)?;
    let mut styles_file = File::create(templates.exports.styles)?;
    let mut responsive_file = File::create(templates.exports.responsive)?;

    page_file.write_all(page.as_bytes())?;
    styles_file.write_all(styles.as_bytes())?;
    responsive_file.write_all(responsive.as_bytes())?;
    
    let config_export = templates.exports.config;
    match File::open(&config_export) {
      Ok(config_file) => {
        let mut buf_reader = BufReader::new(&config_file);
        let mut config_content = String::new();
        buf_reader.read_to_string(&mut config_content)?;

        let mut new_config = File::create(&config_export)?;

        config_content = config_content.replace("// NEXT_ALIAS", &aliases);

        new_config.write_all(config_content.as_bytes())?;
      },
      Err(_) => {
        let mut config_file = File::create(&config_export)?;

        config = config.replace("// NEXT_ALIAS", &aliases);

        config_file.write_all(config.as_bytes())?;
      }
    }

    let styles_index = templates.exports.barrel_styles;
    match File::open(&styles_index) {
      Ok(index_file) => {
        let mut buf_reader = BufReader::new(&index_file);
        let mut index_content = String::new();
        buf_reader.read_to_string(&mut index_content)?;

        if !index_content.contains(&styles_import) {
          index_content = [index_content, styles_import].join("");

          let mut new_index = File::create(&styles_index)?;
    
          new_index.write_all(index_content.as_bytes())?;
        }
      },
      Err(_) => {
        let mut index_file = File::create(&styles_index)?;

        index_file.write_all(styles_import.as_bytes())?;
      }
    }

    if let Some(templates_router) = templates.router {
      if let Some(templates_route) = templates.route {
        let mut router = read_path(
          &template_path,
          templates_router.template,
          templates_router.default,
        );
        let mut route = read_path(
          &template_path,
          templates_route.template,
          templates_route.default,
        );

        route = route.replace("NAME_CAPITAL", &nampe_capital);
        route = route.replace("NAME_LOWER", &name.to_lowercase());
        route = route.replace("NAME", name);

        if let Some(page_import) =templates.imports.page {
          if let Some(router_export) = templates.exports.router {
            match File::open(&router_export) {
              Ok(router_file) => {
                let mut buf_reader = BufReader::new(&router_file);
                let mut router_content = String::new();
                buf_reader.read_to_string(&mut router_content)?;
    
                let mut new_router = File::create(&router_export)?;
    
                router_content = router_content.replace("// ROUTES", &page_import);
                router_content = router_content.replace("// NEXT_ROUTE", &route);
                
                new_router.write_all(router_content.as_bytes())?;
              },
              Err(_) => {
                let mut router_file = File::create(&router_export)?;
    
                router = router.replace("// ROUTES", &page_import);
                router = router.replace("// NEXT_ROUTE", &route);
    
                router_file.write_all(router.as_bytes())?;
              }
            };
          }
        }
      }
    }

    if let Some(templates_i18n) = templates.i18n {
      let mut i18n_context = read_path(
        &template_path,
        templates_i18n.context.template,
        templates_i18n.context.default,
      );
      let mut i18n_locale = read_path(
        &template_path,
        templates_i18n.locale.template,
        templates_i18n.locale.default,
      );

      i18n_locale = i18n_locale.replace("NAME_CAPITAL", &nampe_capital);
      i18n_locale = i18n_locale.replace("NAME_LOWER", &nampe_capital);
      i18n_locale = i18n_locale.replace("NAME", name);

      if let Some(locale_import) = templates.imports.locale {
        if let Some(i18n_export) = templates.exports.i18n {
          match File::open(i18n_export.to_string()) {
            Ok(i18n_file) => {
              let mut buf_reader = BufReader::new(&i18n_file);
              let mut i18n_content = String::new();
              buf_reader.read_to_string(&mut i18n_content)?;

              let mut new_i18n = File::create(&i18n_export)?;

              i18n_content = i18n_content.replace("// NEXT_LOCALE", &locale_import);

              new_i18n.write_all(i18n_content.as_bytes())?;
            },
            Err(_) => {
              let mut i18n_file = File::create(&i18n_export)?;
              
              i18n_context = i18n_context.replace("// NEXT_LOCALE", &locale_import);
              
              i18n_file.write_all(i18n_context.as_bytes())?;
            }
          };

          if let Some(i18n_barrel) = templates.exports.barrel_i18n {
            if let Some(i18n_import) = templates.imports.i18n {
              match File::open(&i18n_barrel) {
                Ok(index) => {
                  let mut buf_reader = BufReader::new(&index);
                  let mut index_content = String::new();
                  buf_reader.read_to_string(&mut index_content)?;
    
                  if index_content.contains("export {};") {
                    index_content = index_content.replace("export {};", "");
                  }
    
                  let mut new_index = File::create(&i18n_barrel)?;
                  let updated_index = [index_content.as_str(), i18n_import.as_str()].concat();
    
                  new_index.write_all(updated_index.as_bytes())?;
                },
                Err(_) => {
                  let mut index = File::create(&i18n_barrel)?;
    
                  index.write_all(i18n_import.as_bytes())?;
                }
              }
            }
          }

          if let Some(i18n_langs) = templates.exports.locales {
            for lang in i18n_langs {
              let mut file_lang = File::create(lang)?;
              file_lang.write_all(i18n_locale.as_bytes())?;
            }
          }
        }
      }
    }

    if let Some(template_props) = templates.proptypes {
      let mut proptypes = read_path(
        &template_path,
        template_props.template,
        template_props.default,
      );

      proptypes = proptypes.replace("NAME_CAPITAL", &nampe_capital);
      proptypes = proptypes.replace("NAME_LOWER", &name.to_lowercase());
      proptypes = proptypes.replace("NAME", name);

      if let Some(export_props) = templates.exports.proptypes {
        let mut proptypes_file = File::create(export_props)?;
        proptypes_file.write_all(proptypes.as_bytes())?;
      }
    }
    
    if let Some(template_tsconfig) = templates.aliases.ts_file {
      if let Some(template_tsaliases) = templates.aliases.ts_aliases {
        let mut tsconfig = read_path(
          &template_path,
          template_tsconfig.template,
          template_tsconfig.default,
        );

        let mut ts_aliases = read_path(
          &template_path,
          template_tsaliases.template,
          template_tsaliases.default,
        );

        ts_aliases = ts_aliases.replace("NAME_CAPITAL", &nampe_capital);
        ts_aliases = ts_aliases.replace("NAME_LOWER", &name.to_lowercase());
        ts_aliases = ts_aliases.replace("NAME", name);

        let tsconfig_export = "./tsconfig.json".to_string();

        match File::open(&tsconfig_export) {
          Ok(tsconfig_file) => {
            let mut buf_reader = BufReader::new(&tsconfig_file);
            let mut tsconfig_content = String::new();
            buf_reader.read_to_string(&mut tsconfig_content)?;

            let mut new_tsconfig = File::create(&tsconfig_export)?;

            tsconfig_content = tsconfig_content.replace("// NEXT_ALIAS", &ts_aliases);

            new_tsconfig.write_all(tsconfig_content.as_bytes())?;
          },
          Err(_) => {
            let mut tsconfig_file = File::create(&tsconfig_export)?;

            tsconfig = tsconfig.replace("// NEXT_ALIAS", &ts_aliases);

            tsconfig_file.write_all(tsconfig.as_bytes())?;
          }
        }
      }
    }

    Ok(())
  }
  pub fn generate_layout(&self, templates: LayoutCreation, tool: &Tool) -> Result<()> {
    let name = self.answers.name.as_str();
    let nampe_capital = change_case(name, None);

    let template_path = match tool {
      Tool::React => templates.react_path(),
      Tool::Svelte => templates.svelte_path(),
      Tool::Vanilla => templates.vanilla_path(),
    };

    let mut styles = read_path(
      &template_path,
      templates.styles.template,
      templates.styles.default
    );
    let mut responsive = read_path(
      &template_path,
      templates.responsive.template,
      templates.responsive.default
    );
    let mut layout = read_path(
      &template_path,
      templates.layout.template,
      templates.layout.default
    );
    if let Some(template_script) = templates.script {
      let script = read_path(
        &template_path,
        template_script.template,
        template_script.default,
      );
      layout = layout.replace("SCRIPT", &script);
    }

    let styles_import = templates.import;

    layout = layout.replace("NAME_CAPITAL", &nampe_capital);
    layout = layout.replace("NAME_LOWER", &name.to_lowercase());
    layout = layout.replace("NAME", name);
    styles = styles.replace("NAME_CAPITAL", &nampe_capital);
    styles = styles.replace("NAME_LOWER", &name.to_lowercase());
    styles = styles.replace("NAME", name);
    responsive = responsive.replace("NAME_CAPITAL", &nampe_capital);
    responsive = responsive.replace("NAME_LOWER", &name.to_lowercase());
    responsive = responsive.replace("NAME", name);

    let mut layout_file = File::create(templates.exports.layout)?;
    let mut styles_file = File::create(templates.exports.styles)?;
    let mut responsive_file = File::create(templates.exports.responsive)?;

    layout_file.write_all(layout.as_bytes())?;
    styles_file.write_all(styles.as_bytes())?;
    responsive_file.write_all(responsive.as_bytes())?;

    let styles_index = templates.exports.barrel_styles;
    match File::open(&styles_index) {
      Ok(index_file) => {
        let mut buf_reader = BufReader::new(&index_file);
        let mut index_content = String::new();
        buf_reader.read_to_string(&mut index_content)?;

        if !index_content.contains(&styles_import) {
          index_content = [index_content, styles_import].join("");

          let mut new_index = File::create(&styles_index)?;
    
          new_index.write_all(index_content.as_bytes())?;
        }
      },
      Err(_) => {
        let mut index_file = File::create(&styles_index)?;

        index_file.write_all(styles_import.as_bytes())?;
      }
    }

    if let Some(template_props) = templates.proptypes {
      let mut proptypes = read_path(
        &template_path,
        template_props.template,
        template_props.default,
      );

      proptypes = proptypes.replace("NAME_CAPITAL", &nampe_capital);
      proptypes = proptypes.replace("NAME_LOWER", &name.to_lowercase());
      proptypes = proptypes.replace("NAME", name);

      if let Some(export_props) = templates.exports.proptypes {
        let mut proptypes_file = File::create(export_props)?;
        proptypes_file.write_all(proptypes.as_bytes())?;
      }
    }

    Ok(())
  }
  pub fn generate_schema(&self) -> Result<()> {
    // For schema generation
    use global::schema::{PROPTYPES, SCHEMA, SCHEMA_TS};
    use global::schema::{NEW_IMPORT, TYPE_EXPORT};

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
    use global::service::{PROPTYPES, SERVICE, SERVICE_TS};
    use global::service::{SERVICE_IMPORT, TYPE_EXPORT, INSTANCES};

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