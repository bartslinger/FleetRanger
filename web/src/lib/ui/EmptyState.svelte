<!--
	@component
	Carbon's empty state: replaces the element that has nothing to show (a table
	loses its headers and pagination). A short title, what will appear and how,
	and optionally one action. Left-aligned.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		body: string;
		action?: Snippet;
	}

	let { title, body, action }: Props = $props();
</script>

<div class="empty">
	<h2>{title}</h2>
	<p>{body}</p>
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

	h2 {
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	p {
		max-width: 40rem;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	.action {
		margin-top: var(--cds-spacing-05);
	}
</style>
