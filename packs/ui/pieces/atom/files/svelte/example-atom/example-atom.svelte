<script lang="ts" module>
	export type ExampleAtomTone = "neutral" | "accent";
</script>

<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLSpanElement> {
		/** Visual emphasis. */
		tone?: ExampleAtomTone;
	}

	let { tone = "neutral", class: className, children, ...rest }: Props = $props();

	// @tfverse if stack.tailwind
	const classes = {
		root: "inline-flex items-center gap-1 rounded-full bg-zinc-100 px-2 py-0.5 text-xs font-semibold text-zinc-700 data-[tone=accent]:bg-indigo-600 data-[tone=accent]:text-white",
	};
	// @tfverse else
	const classes = { root: "root" };
	// @tfverse end
</script>

<span {...rest} class={[classes.root, className]} data-tone={tone}>
	{@render children?.()}
</span>
<!-- @tfverse if !stack.tailwind -->

<style>
	.root {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.125rem 0.5rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
		line-height: 1.5;
		color: var(--ui-neutral-foreground, #3f3f46);
		background: var(--ui-neutral, #f4f4f5);
	}

	.root[data-tone="accent"] {
		color: var(--ui-accent-foreground, #ffffff);
		background: var(--ui-accent, #4f46e5);
	}
</style>
<!-- @tfverse end -->
