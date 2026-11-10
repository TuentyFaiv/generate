use dioxus::prelude::*;

// @tfverse if stack.tailwind
const ROOT: &str = "grid gap-2";
const LABEL: &str = "text-sm font-semibold";
const DESCRIPTION: &str = "text-sm text-zinc-500";
const CONTENT: &str = "flex flex-wrap items-center gap-2";
// @tfverse else
const ROOT: &str = "example-molecule";
const LABEL: &str = "example-molecule__label";
const DESCRIPTION: &str = "example-molecule__description";
const CONTENT: &str = "example-molecule__content";
// @tfverse end

/// Groups atoms under one accessible label, such as a field with its hint.
#[component]
pub fn ExampleMolecule(
	/// Unique id; the label and description ids derive from it.
	#[props(into)]
	id: String,
	/// Names the group for assistive technology.
	#[props(into)]
	label: String,
	/// Extra guidance shown under the label.
	#[props(into)]
	description: Option<String>,
	children: Element,
) -> Element {
	let label_id = format!("{id}-label");
	let labelled_by = label_id.clone();
	let description_id = description.as_ref().map(|_| format!("{id}-description"));
	let described_by = description_id.clone();

	rsx! {
		div {
			id,
			role: "group",
			"aria-labelledby": labelled_by,
			"aria-describedby": described_by,
			class: ROOT,
			span { id: label_id, class: LABEL, {label} }
			if let Some(description) = description {
				p { id: description_id, class: DESCRIPTION, {description} }
			}
			div { class: CONTENT, {children} }
		}
	}
}
