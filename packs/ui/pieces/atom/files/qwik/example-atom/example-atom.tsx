import { type PropsOf, Slot, component$, useStylesScoped$ } from "@builder.io/qwik"; // @tfverse if !stack.tailwind
import { type PropsOf, Slot, component$ } from "@builder.io/qwik"; // @tfverse if stack.tailwind
// @tfverse if !stack.tailwind

import styles from "./example-atom.css?inline";
// @tfverse end

export type ExampleAtomTone = "neutral" | "accent";

export type ExampleAtomProps = PropsOf<"span"> & {
	/** Visual emphasis. */
	tone?: ExampleAtomTone;
};

// @tfverse if stack.tailwind
const classes = {
	root: "inline-flex items-center gap-1 rounded-full bg-zinc-100 px-2 py-0.5 text-xs font-semibold text-zinc-700 data-[tone=accent]:bg-indigo-600 data-[tone=accent]:text-white",
};
// @tfverse else
const classes = { root: "root" };
// @tfverse end

/** The smallest UI building block: an inline, label-sized element. */
export const ExampleAtom = component$<ExampleAtomProps>(({ tone = "neutral", class: className, ...props }) => {
	// @tfverse if !stack.tailwind
	useStylesScoped$(styles);

	// @tfverse end
	return (
		<span {...props} data-tone={tone} class={[classes.root, className]}>
			<Slot />
		</span>
	);
});
