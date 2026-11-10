use dioxus::prelude::*;

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

/// A distinct, labelled region of the page, built from molecules.
#[component]
pub fn ExampleOrganism(
	/// Unique id; the heading id derives from it.
	#[props(into)]
	id: String,
	/// Heading that names the section.
	#[props(into)]
	heading: String,
	/// Controls shown next to the heading.
	actions: Option<Element>,
	children: Element,
) -> Element {
	let heading_id = format!("{id}-heading");
	let labelled_by = heading_id.clone();

	rsx! {
		section { id, "aria-labelledby": labelled_by, class: ROOT,
			header { class: HEADER,
				h2 { id: heading_id, class: TITLE, {heading} }
				if let Some(actions) = actions {
					div { class: ACTIONS, {actions} }
				}
			}
			{children}
		}
	}
}
