pub mod enums;

use self::enums::Tool;

#[derive(Debug)]
pub struct Technologies {
  pub tools: Vec<Tool>,
}

impl Technologies {
  pub fn new() -> Self {

    Self {
      tools: Tool::all(),
    }
  }
}

