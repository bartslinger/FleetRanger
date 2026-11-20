<!--
	@component
	Running text: headings, paragraphs, lists, tables and code, in Carbon's
	expressive body style (body-02), its set for long reading. The text comes
	as a snippet or, for the docs, as HTML.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children, html }: { children?: Snippet; html?: string } = $props();
</script>

<div class="prose">
	{#if html !== undefined}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -- the docs' own Markdown, built into the site -->
		{@html html}
	{:else}
		{@render children?.()}
	{/if}
</div>

<style>
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

	.prose :global(:is(p, ul, ol, table, pre, blockquote)) {
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

	.prose :global(h3) {
		margin-top: var(--cds-spacing-07);
		font-size: var(--cds-heading-02-font-size);
		font-weight: var(--cds-heading-02-font-weight);
		line-height: var(--cds-heading-02-line-height);
		letter-spacing: var(--cds-heading-02-letter-spacing);
	}

	/* Steps: numbered, the numbers where the list's dashes are. */
	.prose :global(ol) {
		margin-inline-start: var(--cds-spacing-06);
		padding: 0;
	}

	.prose :global(ol > li::before) {
		content: none;
	}

	/* Carbon's multi-line code snippet, read-only: without scripts the copy
	   button is left out, which Carbon allows. Long lines scroll rather than
	   wrap, so the box can be scrolled with the keyboard too. */
	.prose :global(pre) {
		overflow-x: auto;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
	}

	.prose :global(pre:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	.prose :global(code) {
		font-family: var(--cds-font-family-mono);
		font-size: var(--cds-code-02-font-size);
		line-height: var(--cds-code-02-line-height);
		letter-spacing: var(--cds-code-02-letter-spacing);
	}

	/* A name or value in a sentence: Carbon's inline snippet without its copy
	   button. */
	.prose :global(:not(pre) > code) {
		padding-inline: var(--cds-spacing-02);
		border-radius: 4px;
		background: var(--cds-field-01);
		white-space: nowrap;
	}

	/* A note beside the text, as `> **Note:** …` in Markdown. */
	.prose :global(blockquote) {
		padding: var(--cds-spacing-05);
		border-inline-start: 3px solid var(--cds-support-info);
		background: var(--cds-layer-01);
	}

	.prose :global(blockquote > :first-child) {
		margin-top: 0;
	}
</style>
