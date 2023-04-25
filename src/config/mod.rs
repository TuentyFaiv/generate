pub struct Paths {
  pub action: String,
  pub store: String,
  pub class: String,
  pub hoc: String,
  pub hook: String,
  pub page: String,
  pub layout: String,
  pub ui: String,
  pub context: String,
  pub service: String,
  pub schema: String,
}

pub struct Config {
  pub paths: Paths,
}

pub fn get_config() -> Config {
  let paths = Paths {
    action: "./src/logic/actions".to_string(),
    store: "./src/logic/stores".to_string(),
    class: "./src/logic/classes".to_string(),
    hoc: "./src/logic/hocs".to_string(),
    hook: "./src/logic/hooks".to_string(),
    page: "./src/routes".to_string(),
    layout: "./src/routes".to_string(),
    ui: "./src/ui".to_string(),
    context: "./src/logic/contexts".to_string(),
    service: "./src/logic/services".to_string(),
    schema: "./src/logic/schemas".to_string(),
  };

  let config = Config {
    paths: paths,
  };

  config
}