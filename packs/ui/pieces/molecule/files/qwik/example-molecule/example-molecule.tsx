import { type PropsOf, Slot, component$, useId, useStylesScoped$ } from "@builder.io/qwik"; // @tfverse if !stack.tailwind
import { type PropsOf, Slot, component$, useId } from "@builder.io/qwik"; // @tfverse if stack.tailwind
// @tfverse if !stack.tailwind

import styles from "./example-molecule.css?inline";
// @tfverse end

export type ExampleMoleculeProps = Omit<PropsOf<"div">, "aria-describedby" | "aria-labelledby" | "role"> & {
	/** Names the group for assistive technology. */
	label: string;
	/** Extra guidance shown under the label. */
	description?: string;
};

// @tfverse if stack.tailwind
const classes = {
	root: "grid gap-2",
	label: "text-sm font-semibold",
	description: "text-sm text-zinc-500",
	content: "flex flex-wrap items-center gap-2",
};
// @tfverse else
const classes = { root: "root", label: "label", description: "description", content: "content" };
// @tfverse end

/** Groups atoms under one accessible label, such as a field with its hint. */
export const ExampleMolecule = component$<ExampleMoleculeProps>(
	({ label, description, class: className, ...props }) => {
		// @tfverse if !stack.tailwind
		useStylesScoped$(styles);
		// @tfverse end
		const id = useId();
		const labelId = `${id}-label`;
		const descriptionId = `${id}-description`;

		return (
			<div
				{...props}
				role="group"
				aria-labelledby={labelId}
				aria-describedby={description ? descriptionId : undefined}
				class={[classes.root, className]}
			>
				<span id={labelId} class={classes.label}>
					{label}
				</span>
				{description && (
					<p id={descriptionId} class={classes.description}>
						{description}
					</p>
				)}
				<div class={classes.content}>
					<Slot />
				</div>
			</div>
		);
	},
);
