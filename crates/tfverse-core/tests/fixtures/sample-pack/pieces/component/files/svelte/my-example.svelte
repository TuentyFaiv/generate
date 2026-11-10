<script lang="ts">
	let { title }: { title: string } = $props();
</script>

<h2 class="my-example">{title}</h2>
<!-- @tfverse if styled -->

<style>
	.my-example {
		display: block;
	}
</style>
<!-- @tfverse end -->
