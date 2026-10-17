<!--
	@component
	The page title, with an optional description and actions.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		description?: string;
		actions?: Snippet;
	}

	let { title, description, actions }: Props = $props();
</script>

<svelte:head>
	<title>{title} · FleetRanger</title>
</svelte:head>

<header>
	<div class="text">
		<h1>{title}</h1>
		{#if description}<p>{description}</p>{/if}
	</div>
	{#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
	header {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: var(--cds-spacing-05);
		margin-bottom: var(--cds-spacing-07);
	}

	.text {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
	}

	h1 {
		font-size: var(--cds-heading-04-font-size);
		font-weight: var(--cds-heading-04-font-weight);
		line-height: var(--cds-heading-04-line-height);
		letter-spacing: var(--cds-heading-04-letter-spacing);
		text-wrap: balance;
	}

	p {
		max-width: 40rem;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-03);
	}
</style>
