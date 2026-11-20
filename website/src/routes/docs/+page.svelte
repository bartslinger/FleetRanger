<!-- The documentation's start: its sections, each with its pages. -->
<script lang="ts">
	import { sections } from '#lib/docs.js';
	import Text from '#lib/Text.svelte';
</script>

<svelte:head>
	<title>Documentation · FleetRanger</title>
	<meta
		name="description"
		content="How to use FleetRanger, and how to run it on your own server."
	/>
</svelte:head>

<h1>Documentation</h1>
<Text>
	{#each sections as section (section.slug)}
		<h2>{section.title}</h2>
		<ul>
			{#each section.pages as p (p.slug)}
				<li><a href="/docs/{section.slug}/{p.slug}">{p.title}</a></li>
			{/each}
		</ul>
	{/each}
</Text>

<style>
	h1 {
		font-size: var(--cds-heading-05-font-size);
		font-weight: var(--cds-heading-05-font-weight);
		line-height: var(--cds-heading-05-line-height);
		letter-spacing: var(--cds-heading-05-letter-spacing);
	}
</style>
