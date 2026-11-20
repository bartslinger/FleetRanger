<!--
	@component
	A page of running text, such as the terms: a title, the date it was last
	changed, and its Text in a column of 8 of the 16 at the left, about 70
	characters wide.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Text from './Text.svelte';

	interface Props {
		title: string;
		/** The date of this version, as shown: "17 November 2026". */
		updated: string;
		children: Snippet;
	}

	let { title, updated, children }: Props = $props();
</script>

<svelte:head>
	<title>{title} · FleetRanger</title>
</svelte:head>

<article>
	<Grid centered>
		<Column md={8} lg={8}>
			<h1>{title}</h1>
			<p class="updated">Last updated {updated}</p>
			<Text>
				{@render children()}
			</Text>
		</Column>
	</Grid>
</article>

<style>
	article {
		padding-block: var(--cds-spacing-10) var(--cds-spacing-12);
	}

	h1 {
		font-size: var(--cds-heading-05-font-size);
		font-weight: var(--cds-heading-05-font-weight);
		line-height: var(--cds-heading-05-line-height);
		letter-spacing: var(--cds-heading-05-letter-spacing);
	}

	.updated {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}
</style>
