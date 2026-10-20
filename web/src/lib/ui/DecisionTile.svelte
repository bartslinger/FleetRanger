<!--
	@component
	A page with one decision, e.g. allowing an app to connect: a white tile
	with a title, a short description, the content, the buttons and a note
	below them, after Carbon's login pattern (nothing else to do). Place it on
	the grid, e.g. centred on six columns. Buttons go in `actions`, primary
	first.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		description?: string;
		/** Below the buttons, e.g. what follows and how to undo it later. */
		helper?: string;
		children: Snippet;
		actions?: Snippet;
	}

	let { title, description, helper, children, actions }: Props = $props();
</script>

<section class="tile layer-01" aria-labelledby="decision-title">
	<div class="heading">
		<h1 id="decision-title">{title}</h1>
		{#if description}<p class="description">{description}</p>{/if}
	</div>
	{@render children()}
	{#if actions}<div class="actions">{@render actions()}</div>{/if}
	{#if helper}<p class="helper">{helper}</p>{/if}
</section>

<style>
	.tile {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-06);
		margin-top: var(--cds-spacing-07);
		padding: var(--cds-spacing-07) var(--cds-spacing-05);
		background: var(--cds-layer-01);
		color: var(--cds-text-primary);
	}

	.heading {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
	}

	h1,
	p {
		margin: 0;
	}

	h1 {
		font-size: var(--cds-heading-04-font-size);
		font-weight: var(--cds-heading-04-font-weight);
		line-height: var(--cds-heading-04-line-height);
		letter-spacing: var(--cds-heading-04-letter-spacing);
	}

	.description {
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	.helper {
		color: var(--cds-text-helper);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}

	/* Side by side, 1px apart as Carbon's button sets. */
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 1px;
	}
</style>
