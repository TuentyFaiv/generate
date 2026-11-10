import { type PropsOf, Slot, component$, useId, useStylesScoped$ } from "@builder.io/qwik"; // @tfverse if !stack.tailwind
import { type PropsOf, Slot, component$, useId } from "@builder.io/qwik"; // @tfverse if stack.tailwind
// @tfverse if !stack.tailwind

import styles from "./example-organism.css?inline";
// @tfverse end

export type ExampleOrganismProps = Omit<PropsOf<"section">, "aria-labelledby"> & {
	/** Heading that names the section. */
	heading: string;
};

// @tfverse if stack.tailwind
const classes = {
	root: "grid gap-4 rounded-xl border border-zinc-200 p-6",
	header: "flex flex-wrap items-center justify-between gap-3",
	title: "text-lg font-semibold",
	actions: "flex gap-2",
};
// @tfverse else
const classes = { root: "root", header: "header", title: "title", actions: "actions" };
// @tfverse end

/** A distinct, labelled region of the page; put controls in the `actions` slot. */
export const ExampleOrganism = component$<ExampleOrganismProps>(({ heading, class: className, ...props }) => {
	// @tfverse if !stack.tailwind
	useStylesScoped$(styles);
	// @tfverse end
	const headingId = useId();

	return (
		<section {...props} aria-labelledby={headingId} class={[classes.root, className]}>
			<header class={classes.header}>
				<h2 id={headingId} class={classes.title}>
					{heading}
				</h2>
				<div class={classes.actions}>
					<Slot name="actions" />
				</div>
			</header>
			<Slot />
		</section>
	);
});
