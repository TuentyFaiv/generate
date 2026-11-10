import { type ComponentProps, type JSX, Show, createUniqueId, splitProps } from "solid-js";
// @tfverse if !stack.tailwind

import styles from "./example-organism.module.css";
// @tfverse end

export type ExampleOrganismProps = Omit<ComponentProps<"section">, "aria-labelledby"> & {
	/** Heading that names the section. */
	heading: JSX.Element;
	/** Controls shown next to the heading. */
	actions?: JSX.Element;
};

// @tfverse if stack.tailwind
const classes = {
	root: "grid gap-4 rounded-xl border border-zinc-200 p-6",
	header: "flex flex-wrap items-center justify-between gap-3",
	title: "text-lg font-semibold",
	actions: "flex gap-2",
};
// @tfverse else
const classes = styles;
// @tfverse end

/** A distinct, labelled region of the page, built from molecules. */
export function ExampleOrganism(props: ExampleOrganismProps) {
	const [local, rest] = splitProps(props, ["heading", "actions", "class", "children"]);
	const headingId = createUniqueId();

	return (
		<section {...rest} aria-labelledby={headingId} class={[classes.root, local.class].filter(Boolean).join(" ")}>
			<header class={classes.header}>
				<h2 id={headingId} class={classes.title}>
					{local.heading}
				</h2>
				<Show when={local.actions}>
					<div class={classes.actions}>{local.actions}</div>
				</Show>
			</header>
			{local.children}
		</section>
	);
}
