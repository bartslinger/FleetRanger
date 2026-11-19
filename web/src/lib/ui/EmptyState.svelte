<!--
	@component
	Carbon's empty state: replaces the element that has nothing to show (a table
	loses its headers and pagination). A short title, what will appear and how,
	and optionally one action. Left-aligned. For a first use that needs setting
	up, numbered steps after the body (Carbon's in-line documentation).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		body: string;
		/** Steps to take, in order, e.g. to get the first items in. */
		steps?: string[];
		action?: Snippet;
		/** The title's heading level: 1 where it is the page's only heading. */
		level?: 1 | 2;
	}

	let { title, body, steps, action, level = 2 }: Props = $props();
</script>

<div class="empty">
	<svelte:element this={`h${level}`} class="title">{title}</svelte:element>
	<p>{body}</p>
	{#if steps}
		<ol>
			{#each steps as step (step)}<li>{step}</li>{/each}
		</ol>
	{/if}
	{#if action}<div class="action">{@render action()}</div>{/if}
</div>

<style>
	.empty {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--cds-spacing-03);
		padding: var(--cds-spacing-09) var(--cds-spacing-05);
		background: var(--cds-layer-01);
	}

	.title {
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	p,
	ol {
		max-width: 40rem;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	/* Carbon's ordered list: numbers in the margin, 8px between steps. */
	ol {
		display: grid;
		gap: var(--cds-spacing-03);
		margin: var(--cds-spacing-03) 0 0;
		padding-inline-start: var(--cds-spacing-06);
		list-style: decimal;
		color: var(--cds-text-primary);
	}

	.action {
		margin-top: var(--cds-spacing-05);
	}

	/* A ghost button hangs: its text lines up with the text above (Carbon). */
	.action :global(.ghost) {
		margin-inline-start: calc(-1 * var(--cds-spacing-05));
	}
</style>
