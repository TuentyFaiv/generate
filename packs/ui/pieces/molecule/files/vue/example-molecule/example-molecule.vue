<script setup lang="ts">
import { useId } from "vue";

const { label, description } = defineProps<{
	/** Names the group for assistive technology. */
	label: string;
	/** Extra guidance shown under the label. */
	description?: string;
}>();

const id = useId();

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
</script>

<template>
	<div
		role="group"
		:aria-labelledby="`${id}-label`"
		:aria-describedby="description ? `${id}-description` : undefined"
		:class="classes.root"
	>
		<span :id="`${id}-label`" :class="classes.label">{{ label }}</span>
		<p v-if="description" :id="`${id}-description`" :class="classes.description">{{ description }}</p>
		<div :class="classes.content"><slot /></div>
	</div>
</template>
<!-- @tfverse if !stack.tailwind -->

<style scoped>
	.root {
		display: grid;
		gap: 0.5rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 600;
	}

	.description {
		margin: 0;
		font-size: 0.875rem;
		color: var(--ui-muted-foreground, #71717a);
	}

	.content {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.5rem;
	}
</style>
<!-- @tfverse end -->
