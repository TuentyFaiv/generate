use yew::prelude::*;

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

/// Props of [`ExampleAtom`].
#[derive(Properties, PartialEq)]
pub struct ExampleAtomProps {
	/// Visual emphasis.
	#[prop_or_default]
	pub tone: ExampleAtomTone,
	/// Extra classes.
	#[prop_or_default]
	pub class: Classes,
	/// Content.
	#[prop_or_default]
	pub children: Html,
}

/// The smallest UI building block: an inline, label-sized element.
#[function_component]
pub fn ExampleAtom(props: &ExampleAtomProps) -> Html {
	html! {
		<span class={classes!(ROOT, props.class.clone())} data-tone={props.tone.as_str()}>
			{ props.children.clone() }
		</span>
	}
}
