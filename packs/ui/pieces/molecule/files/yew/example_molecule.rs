use yew::prelude::*;

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

/// Props of [`ExampleMolecule`].
#[derive(Properties, PartialEq)]
pub struct ExampleMoleculeProps {
	/// Unique id; the label and description ids derive from it.
	pub id: AttrValue,
	/// Names the group for assistive technology.
	pub label: AttrValue,
	/// Extra guidance shown under the label.
	#[prop_or_default]
	pub description: Option<AttrValue>,
	/// Content, usually atoms.
	#[prop_or_default]
	pub children: Html,
}

/// Groups atoms under one accessible label, such as a field with its hint.
#[function_component]
pub fn ExampleMolecule(props: &ExampleMoleculeProps) -> Html {
	let label_id = format!("{}-label", props.id);
	let description_id = props.description.as_ref().map(|_| format!("{}-description", props.id));

	html! {
		<div
			id={props.id.clone()}
			role="group"
			aria-labelledby={label_id.clone()}
			aria-describedby={description_id.clone()}
			class={ROOT}
		>
			<span id={label_id} class={LABEL}>{ &props.label }</span>
			if let Some(description) = &props.description {
				<p id={description_id} class={DESCRIPTION}>{ description }</p>
			}
			<div class={CONTENT}>{ props.children.clone() }</div>
		</div>
	}
}
