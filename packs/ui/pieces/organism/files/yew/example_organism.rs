use yew::prelude::*;

// @tfverse if stack.tailwind
const ROOT: &str = "grid gap-4 rounded-xl border border-zinc-200 p-6";
const HEADER: &str = "flex flex-wrap items-center justify-between gap-3";
const TITLE: &str = "text-lg font-semibold";
const ACTIONS: &str = "flex gap-2";
// @tfverse else
const ROOT: &str = "example-organism";
const HEADER: &str = "example-organism__header";
const TITLE: &str = "example-organism__title";
const ACTIONS: &str = "example-organism__actions";
// @tfverse end

/// Props of [`ExampleOrganism`].
#[derive(Properties, PartialEq)]
pub struct ExampleOrganismProps {
	/// Unique id; the heading id derives from it.
	pub id: AttrValue,
	/// Heading that names the section.
	pub heading: AttrValue,
	/// Controls shown next to the heading.
	#[prop_or_default]
	pub actions: Option<Html>,
	/// Section content.
	#[prop_or_default]
	pub children: Html,
}

/// A distinct, labelled region of the page, built from molecules.
#[function_component]
pub fn ExampleOrganism(props: &ExampleOrganismProps) -> Html {
	let heading_id = format!("{}-heading", props.id);

	html! {
		<section id={props.id.clone()} aria-labelledby={heading_id.clone()} class={ROOT}>
			<header class={HEADER}>
				<h2 id={heading_id} class={TITLE}>{ &props.heading }</h2>
				if let Some(actions) = &props.actions {
					<div class={ACTIONS}>{ actions.clone() }</div>
				}
			</header>
			{ props.children.clone() }
		</section>
	}
}
