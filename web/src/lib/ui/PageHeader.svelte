<!--
	@component
	Carbon's page header: a band across the content area with the page title,
	an optional description and page actions, on the grid.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Column from './Column.svelte';
	import Grid from './Grid.svelte';

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
	<Grid>
		<Column>
			<div class="content">
				<div class="text">
					<h1>{title}</h1>
					{#if description}<p>{description}</p>{/if}
				</div>
				{#if actions}<div class="actions">{@render actions()}</div>{/if}
			</div>
		</Column>
	</Grid>
</header>

<style>
	header {
		margin-bottom: var(--cds-spacing-07);
		padding-block: var(--cds-spacing-06);
		background: var(--cds-layer-01);
		border-bottom: 1px solid var(--cds-border-subtle-01);
	}

	.content {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: var(--cds-spacing-05);
	}

	.text {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		max-width: 40rem;
	}

	h1 {
		font-size: var(--cds-heading-04-font-size);
		font-weight: var(--cds-heading-04-font-weight);
		line-height: var(--cds-heading-04-line-height);
		letter-spacing: var(--cds-heading-04-letter-spacing);
		text-wrap: balance;
	}

	p {
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
