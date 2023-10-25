use anyhow::Result;

use crate::cli::enums::{Lang, Styles};
use crate::cli::utils::{input, sure};

pub fn repository_question() -> Result<String> {
  Ok(input("Repository boilerplates:", "git@github.com:TuentyFaiv")?)
}

pub fn i18n_question() -> Result<bool> {
  sure("Do you want to use i18n? (internationalization)")
}

pub fn lang_question() -> Result<Lang> {
  Ok(Lang::parse(&input("Do you want to use TypeScript or JavaScript?", "typescript")?))
}

pub fn styles_question() -> Result<Styles> {
  Ok(Styles::parse(&input("What kind of styles do you want to use?", "emotion")?))
}