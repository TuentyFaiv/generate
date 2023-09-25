use clap::Parser;

use crate::utils::{change_case, transform};

use super::enums::{ArchType, Tool, Lang, Styles};

#[derive(Parser, Clone, Debug)]
#[command(author, version, about)]
pub struct Args {
	/// Project name
	pub name: Option<String>,
	/// Template tool to choose
	#[arg(short, long)]
	pub tool: Option<String>,
	/// Frontend piece or project template
  #[arg(short, long)]
	pub arch: Option<String>,
	/// TypeScript or JavaScript
  #[arg(short, long)]
	pub language: Option<String>,
	/// Styles type
  #[arg(short, long)]
	pub styles: Option<String>,
	/// Template path to generate
  #[arg(short, long)]
	pub path: Option<String>,
	/// Accept all questions
  #[arg(short = 'y', long)]
	pub sure: bool,
	/// Set configuration
  #[arg(short = 'g', long)]
	pub global: bool,
  /// Config file to customize this CLI
  #[arg(short, long, value_name = "FILE")]
	pub config: Option<String>,
}


#[derive(Clone, Debug)]
pub struct Answers {
  pub name: AnswersName,
  pub path: String,
  pub tool: Tool,
  pub tool_type: Option<String>,
  pub language: Lang,
  pub styles: Styles,
  pub arch: ArchType,
  pub i18n: bool,
  pub accept: bool,
}

#[derive(Clone, Debug)]
pub struct AnswersName {
  pub camel: String,
  pub pascal: String,
  pub dash: String,
  pub constant: String,
  pub snake: String,
  pub lower: String,
  pub original: String,
  pub namespace: String,
}

impl AnswersName {
  // someName // CamelCase
  // SomeName // PascalCase
  // some_name // snake_case
  // Some_Name // Dash_Case
  // SOME_NAME // CONSTANT_CASE
  pub fn new(name: &String) -> Self {
    let pascal = change_case(&transform(&name, None), None);
    let dash = transform(&name, Some("dash"));

    Self {
      camel: change_case(&pascal, Some("camel")),
      lower: pascal.to_lowercase(),
      pascal,
      constant: dash.to_uppercase(),
      snake: dash.to_lowercase(),
      dash,
      original: name.clone(),
      namespace: String::new(),
    }
  }

  pub fn change(&mut self, to: &String) {
    let pascal = change_case(&transform(&to, None), None);
    let dash = transform(&to, Some("dash"));

    self.camel = change_case(&pascal, Some("camel"));
    self.lower = pascal.to_lowercase();
    self.pascal = pascal;
    self.constant = dash.to_uppercase();
    self.snake = dash.to_lowercase();
    self.dash = dash;
    self.original = to.clone();
  }

  pub fn set_namespace(&mut self, path: &String) {
    let namespace = path.split('/')
      .collect::<Vec<&str>>().last().unwrap()
      .to_owned().to_owned();
    self.namespace = namespace;
  }
}

pub struct AnswersToolType {
  pub tool_type: Option<String>,
  pub language: Lang,
}

pub struct QuestionToolType<'a> {
  pub prompt: &'a str,
  pub tools: Vec<String>,
}
