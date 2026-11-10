<script lang="ts">
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "aria-describedby" | "aria-labelledby" | "role"> {
		/** Names the group for assistive technology. */
		label: string;
		/** Extra guidance shown under the label. */
		description?: string;
	}

	let { label, description, class: className, children, ...rest }: Props = $props();
	const id = $props.id();

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

<div
	{...rest}
	role="group"
	aria-labelledby="{id}-label"
	aria-describedby={description ? `${id}-description` : undefined}
	class={[classes.root, className]}
>
	<span id="{id}-label" class={classes.label}>{label}</span>
	{#if description}
		<p id="{id}-description" class={classes.description}>{description}</p>
	{/if}
	<div class={classes.content}>
		{@render children?.()}
	</div>
</div>
<!-- @tfverse if !stack.tailwind -->

<style>
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
