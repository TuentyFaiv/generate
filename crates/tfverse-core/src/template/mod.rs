//! tfverse's template engine: real code, comment directives and case-aware replacement.

mod blocks;
mod expr;
mod replace;

pub use blocks::BlockError;
pub(crate) use blocks::conditions;
pub(crate) use expr::Expr;
pub use expr::ExprError;
pub use replace::ReplaceError;
pub(crate) use replace::{Example, Replacer};

use crate::model::Values;

const BINARY_SNIFF_LEN: usize = 8000;

/// Whether `bytes` are copied verbatim instead of rendered.
pub(crate) fn is_binary(bytes: &[u8]) -> bool {
	bytes[..bytes.len().min(BINARY_SNIFF_LEN)].contains(&0) || std::str::from_utf8(bytes).is_err()
}

/// Renders one text template.
pub(crate) fn render(text: &str, values: &Values, replacer: &Replacer) -> Result<String, BlockError> {
	Ok(replacer.apply(&blocks::process(text, values)?))
}
