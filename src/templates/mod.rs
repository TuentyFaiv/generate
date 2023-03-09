use std::collections::HashMap;

pub mod make;
pub mod svelte;
pub mod react;
pub mod vanilla;

pub fn get_templates<'a>() -> HashMap<&'a str, &'a str>{
	let not_yet = "err-notimplemented";
	let mut templates = HashMap::<&str, &str>::new();
	// React template
	templates.insert("react-next-ts-atomic", "repo-DittoNext");
	templates.insert("react-next-atomic", not_yet);
	templates.insert("react-remix-ts-atomic", not_yet);
	templates.insert("react-remix-atomic", not_yet);
	templates.insert("react-typescript-atomic", "repo-DittoReact");
	templates.insert("react-javascript-atomic", not_yet);
	// React library
	templates.insert("react-typescript-library", not_yet);
	templates.insert("react-javascript-library", not_yet);
	// React component
	templates.insert("react-typescript-component", not_yet);
	templates.insert("react-javascript-component", not_yet);

	// Svelte template
	templates.insert("svelte-sveltekit-ts-atomic", "repo-DittoSvelte");
	templates.insert("svelte-sveltekit-atomic", not_yet);
	templates.insert("svelte-typescript-atomic", not_yet);
	templates.insert("svelte-javascript-atomic", not_yet);
	// Svelte library
	templates.insert("svelte-sveltekit-ts-library", not_yet);
	templates.insert("svelte-sveltekit-library", not_yet);
	templates.insert("svelte-webcomponent-ts-library", not_yet);
	templates.insert("svelte-webcomponent-library", not_yet);
	templates.insert("svelte-typescript-library", not_yet);
	templates.insert("svelte-javascript-library", not_yet);
	// Svelte component
	templates.insert("svelte-typescript-component", "component-svelte");
	templates.insert("svelte-javascript-component", "component-svelte");

	// Vanilla template
	templates.insert("vanilla-typescript-atomic", not_yet);
	templates.insert("vanilla-javascript-atomic", not_yet);
	// Vanilla library
	templates.insert("vanilla-typescript-library", not_yet);
	templates.insert("vanilla-javascript-library", not_yet);
	// Vanilla component
	templates.insert("vanilla-typescript-component", not_yet);
	templates.insert("vanilla-javascript-component", not_yet);

	return templates;
}
