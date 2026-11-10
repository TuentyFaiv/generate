use leptos::prelude::*;

// @tfverse if stack.tailwind
const ROOT: &str = "inline-flex items-center gap-1 rounded-full bg-zinc-100 px-2 py-0.5 text-xs font-semibold text-zinc-700 data-[tone=accent]:bg-indigo-600 data-[tone=accent]:text-white";
// @tfverse else
const ROOT: &str = "example-atom";
// @tfverse end

/// Visual emphasis of an [`ExampleAtom`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExampleAtomTone {
	/// The default, quiet look.
	#[default]
	Neutral,
	/// Draws attention.
	Accent,
}

impl ExampleAtomTone {
	/// The `data-tone` attribute value.
	#[must_use]
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Neutral => "neutral",
			Self::Accent => "accent",
		}
	}
}

/// The smallest UI building block: an inline, label-sized element.
#[component]
pub fn ExampleAtom(
	/// Visual emphasis.
	#[prop(optional)]
	tone: ExampleAtomTone,
	/// Extra classes.
	#[prop(optional, into)]
	class: String,
	children: Children,
) -> impl IntoView {
	view! {
		<span class=classes(ROOT, &class) data-tone=tone.as_str()>
			{children()}
		</span>
	}
}

fn classes(base: &str, extra: &str) -> String {
	if extra.is_empty() {
		base.to_owned()
	} else {
		format!("{base} {extra}")
	}
}
