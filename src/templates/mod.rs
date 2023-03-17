use std::collections::HashMap;

pub mod svelte;
pub mod react;
pub mod vanilla;
pub mod global;

pub fn get_templates<'a>() -> HashMap<&'a str, &'a str>{
	let not_yet = "err-notimplemented";
	let mut templates = HashMap::<&str, &str>::new();

	// React template
	templates.insert("react-next-ts-atomic", "repo-DittoNext");
	templates.insert("react-next-atomic", not_yet);
	templates.insert("react-remix-ts-atomic", not_yet);
	templates.insert("react-remix-atomic", not_yet);
	templates.insert("react-native-ts-atomic", not_yet);
	templates.insert("react-native-atomic", not_yet);
	templates.insert("react-typescript-atomic", "repo-DittoReact");
	templates.insert("react-javascript-atomic", not_yet);
	// React library
	templates.insert("react-typescript-library", "library-DittoReact");
	templates.insert("react-javascript-library", not_yet);
	// React component
	templates.insert("react-typescript-component", "component-react");
	templates.insert("react-javascript-component", "component-react");
	// React hoc
	templates.insert("react-typescript-hoc", "hoc-react");
	templates.insert("react-javascript-hoc", "hoc-react");
	// React hook
	templates.insert("react-typescript-hook", "hook-react");
	templates.insert("react-javascript-hook", "hook-react");
	// React context
	templates.insert("react-typescript-context", "context-react");
	templates.insert("react-javascript-context", "context-react");
	// React page
	templates.insert("react-typescript-page", "page-react");
	templates.insert("react-javascript-page", "page-react");
	// React layout
	templates.insert("react-typescript-layout", "layout-react");
	templates.insert("react-javascript-layout", "layout-react");
	// React service
	templates.insert("react-typescript-service", "service-react");
	templates.insert("react-javascript-service", "service-react");
	// React schema
	templates.insert("react-typescript-schema", "schema-react");
	templates.insert("react-javascript-schema", "schema-react");

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
	// Svelte page
	templates.insert("svelte-typescript-page", not_yet);
	templates.insert("svelte-javascript-page", not_yet);
	// Svelte layout
	templates.insert("svelte-typescript-layout", not_yet);
	templates.insert("svelte-javascript-layout", not_yet);
	// Svelte action
	templates.insert("svelte-typescript-action", not_yet);
	templates.insert("svelte-javascript-action", not_yet);
	// Svelte store
	templates.insert("svelte-typescript-store", not_yet);
	templates.insert("svelte-javascript-store", not_yet);
	// Svelte service
	templates.insert("svelte-typescript-service", "service-svelte");
	templates.insert("svelte-javascript-service", "service-svelte");
	// Svelte schema
	templates.insert("svelte-typescript-schema", "schema-svelte");
	templates.insert("svelte-javascript-schema", "schema-svelte");

	// Vanilla template
	templates.insert("vanilla-typescript-atomic", not_yet);
	templates.insert("vanilla-javascript-atomic", not_yet);
	// Vanilla library
	templates.insert("vanilla-typescript-library", "library-DittoVanilla");
	templates.insert("vanilla-javascript-library", not_yet);
	// Vanilla component
	templates.insert("vanilla-typescript-component", not_yet);
	templates.insert("vanilla-javascript-component", not_yet);
	// Vanilla page
	templates.insert("vanilla-typescript-page", not_yet);
	templates.insert("vanilla-javascript-page", not_yet);
	// Vanilla layout
	templates.insert("vanilla-typescript-layout", not_yet);
	templates.insert("vanilla-javascript-layout", not_yet);
	// Vanilla class
	templates.insert("vanilla-typescript-class", not_yet);
	templates.insert("vanilla-javascript-class", not_yet);
	// Svelte service
	templates.insert("vanilla-typescript-service", "service-vanilla");
	templates.insert("vanilla-javascript-service", "service-vanilla");
	// Svelte schema
	templates.insert("vanilla-typescript-schema", "schema-vanilla");
	templates.insert("vanilla-javascript-schema", "schema-vanilla");

	return templates;
}
