import { type ComponentProps, splitProps } from "solid-js";
// @tfverse if !stack.tailwind

import styles from "./example-atom.module.css";
// @tfverse end

export type ExampleAtomTone = "neutral" | "accent";

export type ExampleAtomProps = ComponentProps<"span"> & {
	/** Visual emphasis. */
	tone?: ExampleAtomTone;
};

// @tfverse if stack.tailwind
const classes = {
	root: "inline-flex items-center gap-1 rounded-full bg-zinc-100 px-2 py-0.5 text-xs font-semibold text-zinc-700 data-[tone=accent]:bg-indigo-600 data-[tone=accent]:text-white",
};
// @tfverse else
const classes = styles;
// @tfverse end

/** The smallest UI building block: an inline, label-sized element. */
export function ExampleAtom(props: ExampleAtomProps) {
	const [local, rest] = splitProps(props, ["tone", "class"]);

	return (
		<span {...rest} data-tone={local.tone ?? "neutral"} class={[classes.root, local.class].filter(Boolean).join(" ")} />
	);
}
