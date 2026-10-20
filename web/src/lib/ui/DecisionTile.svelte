<!--
	@component
	A page with one decision, e.g. allowing an app to connect: a white tile
	with a title, a description, the content and the buttons, after Carbon's
	login pattern (centred, nothing else to do). Buttons go in `actions`,
	primary first; they stack on phones with the primary on top.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		description?: string;
		/** Facts the decision rests on, as label and value, e.g. who asks. */
		facts?: { label: string; value: string }[];
		/** Below the content, e.g. how to undo the decision later. */
		helper?: string;
		children: Snippet;
		actions?: Snippet;
	}

	let { title, description, facts = [], helper, children, actions }: Props = $props();
</script>

<section class="tile layer-01" aria-labelledby="decision-title">
	<div class="heading">
		<h1 id="decision-title">{title}</h1>
		{#if description}<p class="description">{description}</p>{/if}
	</div>
	{#if facts.length > 0}
		<dl class="facts">
			{#each facts as fact (fact.label)}
				<div>
					<dt>{fact.label}</dt>
					<dd>{fact.value}</dd>
				</div>
			{/each}
		</dl>
	{/if}
	{@render children()}
	{#if helper}<p class="helper">{helper}</p>{/if}
	{#if actions}<div class="actions">{@render actions()}</div>{/if}
</section>

<style>
	.tile {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-06);
		max-width: 40rem;
		margin: var(--cds-spacing-07) auto 0;
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
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	/* As facts beside a flight's map: a label above its value, 16px apart. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0;
	}

	dt {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		overflow-wrap: anywhere;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
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
