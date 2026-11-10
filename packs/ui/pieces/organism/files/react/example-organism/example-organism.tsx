import { type ComponentProps, type ReactNode, useId } from "react";
// @tfverse if !stack.tailwind

import styles from "./example-organism.module.css";
// @tfverse end

export type ExampleOrganismProps = Omit<ComponentProps<"section">, "aria-labelledby"> & {
	/** Heading that names the section. */
	heading: ReactNode;
	/** Controls shown next to the heading. */
	actions?: ReactNode;
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
export function ExampleOrganism({ heading, actions, className, children, ...props }: ExampleOrganismProps) {
	const headingId = useId();

	return (
		<section {...props} aria-labelledby={headingId} className={[classes.root, className].filter(Boolean).join(" ")}>
			<header className={classes.header}>
				<h2 id={headingId} className={classes.title}>
					{heading}
				</h2>
				{actions ? <div className={classes.actions}>{actions}</div> : null}
			</header>
			{children}
		</section>
	);
}
