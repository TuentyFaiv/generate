import { type ComponentProps, type JSX, Show, createUniqueId, splitProps } from "solid-js";
// @tfverse if !stack.tailwind

import styles from "./example-molecule.module.css";
// @tfverse end

export type ExampleMoleculeProps = Omit<ComponentProps<"div">, "aria-describedby" | "aria-labelledby" | "role"> & {
	/** Names the group for assistive technology. */
	label: JSX.Element;
	/** Extra guidance shown under the label. */
	description?: JSX.Element;
};

// @tfverse if stack.tailwind
const classes = {
	root: "grid gap-2",
	label: "text-sm font-semibold",
	description: "text-sm text-zinc-500",
	content: "flex flex-wrap items-center gap-2",
};
// @tfverse else
const classes = styles;
// @tfverse end

/** Groups atoms under one accessible label, such as a field with its hint. */
export function ExampleMolecule(props: ExampleMoleculeProps) {
	const [local, rest] = splitProps(props, ["label", "description", "class", "children"]);
	const id = createUniqueId();
	const labelId = `${id}-label`;
	const descriptionId = `${id}-description`;

	return (
		<div
			{...rest}
			role="group"
			aria-labelledby={labelId}
			aria-describedby={local.description ? descriptionId : undefined}
			class={[classes.root, local.class].filter(Boolean).join(" ")}
		>
			<span id={labelId} class={classes.label}>
				{local.label}
			</span>
			<Show when={local.description}>
				<p id={descriptionId} class={classes.description}>
					{local.description}
				</p>
			</Show>
			<div class={classes.content}>{local.children}</div>
		</div>
	);
}
