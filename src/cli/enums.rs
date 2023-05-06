#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ArchType {
  Atomic,
  Library,
  Component,
  Hoc,
  Hook,
  Context,
  Layout,
  Page,
  Service,
  Schema,
  Action,
  Store,
  Class,
}

impl ArchType {
  pub fn parse(name: &str) -> ArchType {
    match name {
      "atomic" => ArchType::Atomic,
      "library" => ArchType::Library,
      "component" => ArchType::Component,
      "hoc" => ArchType::Hoc,
      "hook" => ArchType::Hook,
      "context" => ArchType::Context,
      "layout" => ArchType::Layout,
      "page" => ArchType::Page,
      "service" => ArchType::Service,
      "schema" => ArchType::Schema,
      "action" => ArchType::Action,
      "store" => ArchType::Store,
      "class" => ArchType::Class,
      _ => ArchType::Atomic,
    }
  }
}


