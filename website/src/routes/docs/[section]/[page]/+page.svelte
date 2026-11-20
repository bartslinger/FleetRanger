<script lang="ts">
	import Text from '#lib/Text.svelte';

	let { data } = $props();
</script>

<svelte:head>
	<title>{data.page.title} · FleetRanger</title>
</svelte:head>

<article>
	<h1>{data.page.title}</h1>
	<Text html={data.page.html} />
</article>

<style>
	h1 {
		font-size: var(--cds-heading-05-font-size);
		font-weight: var(--cds-heading-05-font-weight);
		line-height: var(--cds-heading-05-line-height);
		letter-spacing: var(--cds-heading-05-letter-spacing);
	}
</style>
