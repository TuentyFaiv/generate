use console::Emoji;

pub static TOOLS: &[&str] = &["react", "svelte", "vanilla"];
pub static TOOLS_REACT: &[&str] = &["next", "next-ts", "remix", "remix-ts", "native", "native-ts"];
pub static TOOLS_SVELTE: &[&str] = &["sveltekit", "sveltekit-ts"];
pub static TOOLS_WEBCOMPONENTS: &[&str] = &["webcomponent", "webcomponent-ts"];
pub static TOOLS_BASE: &[&str] = &["javascript", "typescript"];

pub static ARCHS: &[&str] = &["atomic", "library", "component", "service", "schema", "page", "layout"];
pub static ARCHS_REACT: &[&str] = &["hoc", "hook", "context"];
pub static ARCHS_SVELTE: &[&str] = &["action", "store"];
pub static ARCHS_VANILLA: &[&str] = &["class"];
pub static ARCHS_TYPE_COMPONENT: &[&str] = &["atoms", "molecules", "organisms", "normal"];

pub static NOT_IMPLEMENTED: Emoji<'_, '_> = Emoji("👻 ", "");
pub static DONE: Emoji<'_, '_> = Emoji("😎 ", "");
pub static OK: Emoji<'_, '_> = Emoji("🥳 ", "");