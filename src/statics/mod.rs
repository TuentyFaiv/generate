use console::Emoji;

use crate::config::file::RepositoryTool;

pub static TOOLS: &[&str] = &["react", "svelte", "vanilla"];
pub static TOOLS_REACT: &[(&str, Option<RepositoryTool>)] = &[
  ("base", Some(RepositoryTool {
    name: "BoilerplateReact",
    project: "main",
    library: Some("library"),
  })),
  ("next", Some(RepositoryTool {
    name: "BoilerplateNext",
    project: "main",
    library: None
  })),
  ("remix", None),
  ("native", None),
  ("gatsby", Some(RepositoryTool {
    name: "BoilerplateGatsby",
    project: "main",
    library: None
  })),
];
pub static TOOLS_SVELTE: &[(&str, Option<RepositoryTool>)] = &[
  ("base", None),
  ("sveltekit", Some(RepositoryTool {
    name: "BoilerplateSveltekit",
    project: "main",
    library: Some("library")
  }))
];
pub static TOOLS_VANILLA: &[(&str, Option<RepositoryTool>)] = &[
  ("base", None),
];
pub static TOOLS_WEBCOMPONENTS: &[&str] = &["webcomponent"];//, "webcomponent-ts"];
pub static TOOLS_COMPONENTS: &[&str] = &["atoms", "molecules", "organisms", "custom"];
pub static LANGS: &[&str] = &["javascript", "typescript"];

pub static ARCHS: &[&str] = &["project", "library", "component", "service", "schema", "page", "layout"];
pub static ARCHS_REACT: &[&str] = &["hoc", "hook", "context"];
pub static ARCHS_SVELTE: &[&str] = &["action", "store"];
pub static ARCHS_VANILLA: &[&str] = &["class"];

pub static NOT_IMPLEMENTED: Emoji<'_, '_> = Emoji("👻 ", "");
pub static DONE: Emoji<'_, '_> = Emoji("😎 ", "");
pub static OK: Emoji<'_, '_> = Emoji("🥳 ", "");