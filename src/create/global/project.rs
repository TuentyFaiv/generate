use anyhow::Result;
use console::style;

use crate::cli::questions::Answers;
use crate::cli::{done, msg};
use crate::templates::{global};

pub fn make(answers: &Answers, template: &Vec<&str>, is_library: bool) -> Result<()> {
  let tool = answers.tool.as_str();
  let name = answers.name.as_str();
  let path = answers.path.as_str();
  let arch = answers.arch.as_str();

  let repository = *template.get(1).unwrap();

  let result = match tool {
    _ => {
      global::project::generate(repository, name, path, tool, arch, is_library);
      true
    }
  };

  if result {
    done();
    msg(&format!("{}", style(format!("Move to {path} and start a new universe")).cyan()));
  }

  Ok(())
}
