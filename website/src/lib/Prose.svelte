<!--
	@component
	A page of running text, such as the terms: a title, the date it was last
	changed, and sections of headings, paragraphs, lists and a plain table.
	Carbon's expressive body style (body-02), its set for long reading, in a
	column of 8 of the 16 at the left, about 70 characters wide.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';

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
			<div class="prose">
				{@render children()}
			</div>
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

	.prose {
		margin-top: var(--cds-spacing-06);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
		letter-spacing: var(--cds-body-02-letter-spacing);
	}

	/* The page's text arrives as a snippet, so its elements are styled from here. */
	.prose :global(h2) {
		margin-top: var(--cds-spacing-09);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	.prose :global(:is(p, ul, table)) {
		margin-top: var(--cds-spacing-05);
	}

	.prose :global(> p:first-child) {
		margin-top: 0;
	}

	/* Carbon's unordered list: an en dash before each item, no space between
	   items. */
	.prose :global(ul) {
		margin-inline-start: var(--cds-spacing-05);
		padding: 0;
		list-style: none;
	}

	.prose :global(li) {
		position: relative;
		padding-inline-start: var(--cds-spacing-02);
	}

	.prose :global(li::before) {
		position: absolute;
		inset-inline-start: calc(-1 * var(--cds-spacing-05));
		content: '–';
	}

	/* Carbon's inline link: underlined, the size of the text around it. */
	.prose :global(a) {
		color: var(--cds-link-primary);
		text-decoration: underline;
	}

	.prose :global(a:visited) {
		color: var(--cds-link-visited);
	}

	.prose :global(a:hover) {
		color: var(--cds-link-primary-hover);
	}

	.prose :global(a:active) {
		color: var(--cds-text-primary);
	}

	.prose :global(a:is(:focus, :active)) {
		outline: 1px solid var(--cds-focus);
	}

	/* A plain table for a few fixed rows, in the data table's colours. */
	.prose :global(table) {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-align: start;
	}

	.prose :global(:is(th, td)) {
		padding: var(--cds-spacing-04) var(--cds-spacing-05);
		border-bottom: 1px solid var(--cds-border-subtle-01);
		text-align: start;
		vertical-align: top;
	}

	.prose :global(th) {
		background: var(--cds-layer-accent-01);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}
</style>
