<script setup lang="ts">
import { useId } from "vue";

const { heading } = defineProps<{
	/** Heading that names the section. */
	heading: string;
}>();

const slots = defineSlots<{
	/** Section content. */
	default?(): unknown;
	/** Controls shown next to the heading. */
	actions?(): unknown;
}>();

const headingId = useId();

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
</script>

<template>
	<section :aria-labelledby="headingId" :class="classes.root">
		<header :class="classes.header">
			<h2 :id="headingId" :class="classes.title">{{ heading }}</h2>
			<div v-if="slots.actions" :class="classes.actions"><slot name="actions" /></div>
		</header>
		<slot />
	</section>
</template>
<!-- @tfverse if !stack.tailwind -->

<style scoped>
	.root {
		display: grid;
		gap: 1rem;
		padding: 1.5rem;
		border: 1px solid var(--ui-border, #e4e4e7);
		border-radius: 0.75rem;
	}

	.header {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
	}

	.title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
	}

	.actions {
		display: flex;
		gap: 0.5rem;
	}
</style>
<!-- @tfverse end -->
