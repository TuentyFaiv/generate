use clap::Parser;

use super::enums::{ArchType, Tool};

#[derive(Parser, Clone, Debug)]
#[command(author, version, about)]
pub struct Args {
	/// Project name
	pub name: Option<String>,
	/// Template tool to choose
	#[arg(short, long)]
	pub tool: Option<String>,
	/// Frontend architecture
  #[arg(short, long)]
	pub arch: Option<String>,
	/// Template path to generate
  #[arg(short, long)]
	pub path: Option<String>,
  /// Config file to customize this CLI
  #[arg(short, long)]
	pub config: Option<String>,
}


#[derive(Clone, Debug)]
pub struct Answers {
  pub name: String,
  pub path: String,
  pub tool: Tool,
  pub tool_type: Option<String>,
  pub language: String,
  pub arch: ArchType,
  pub accept: bool,
}

pub struct AnswersName {
  pub name: String,
  pub namespace: String,
  pub name_lower: String,
}

pub struct AnswersToolType {
  pub tool_type: Option<String>,
  pub language: String,
}

pub struct AnswersPath {
  pub path: String,
  pub name: String,
}

pub struct QuestionToolType<'a> {
  pub prompt: &'a str,
  pub tools: Vec<String>,
}
