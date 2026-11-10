<script lang="ts">
export type ExampleAtomTone = "neutral" | "accent";
</script>

<script setup lang="ts">
const { tone = "neutral" } = defineProps<{
	/** Visual emphasis. */
	tone?: ExampleAtomTone;
}>();

// @tfverse if stack.tailwind
const classes = {
	root: "inline-flex items-center gap-1 rounded-full bg-zinc-100 px-2 py-0.5 text-xs font-semibold text-zinc-700 data-[tone=accent]:bg-indigo-600 data-[tone=accent]:text-white",
};
// @tfverse else
const classes = { root: "root" };
// @tfverse end
</script>

<template>
	<span :class="classes.root" :data-tone="tone"><slot /></span>
</template>
<!-- @tfverse if !stack.tailwind -->

<style scoped>
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
