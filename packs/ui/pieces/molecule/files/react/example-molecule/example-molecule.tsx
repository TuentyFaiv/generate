import { type ComponentProps, type ReactNode, useId } from "react";
// @tfverse if !stack.tailwind

import styles from "./example-molecule.module.css";
// @tfverse end

export type ExampleMoleculeProps = Omit<ComponentProps<"div">, "aria-describedby" | "aria-labelledby" | "role"> & {
	/** Names the group for assistive technology. */
	label: ReactNode;
	/** Extra guidance shown under the label. */
	description?: ReactNode;
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
export function ExampleMolecule({ label, description, className, children, ...props }: ExampleMoleculeProps) {
	const id = useId();
	const labelId = `${id}-label`;
	const descriptionId = description ? `${id}-description` : undefined;

	return (
		<div
			{...props}
			role="group"
			aria-labelledby={labelId}
			aria-describedby={descriptionId}
			className={[classes.root, className].filter(Boolean).join(" ")}
		>
			<span id={labelId} className={classes.label}>
				{label}
			</span>
			{description ? (
				<p id={descriptionId} className={classes.description}>
					{description}
				</p>
			) : null}
			<div className={classes.content}>{children}</div>
		</div>
	);
}
