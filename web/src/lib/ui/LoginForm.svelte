<!--
	@component
	Carbon's login pattern, default style: a title, a short description, then
	messages and the login action, with optional helper text below. It places
	itself on the grid: the form on four columns, as fields should be as wide
	as their content (Carbon's forms pattern), and the title on more, so a
	longer one stays on one line, on the same left edge (user's choice on the
	carbon-expert's advice, 2026-11-16; Carbon sets no width for login pages).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Column from './Column.svelte';
	import Grid from './Grid.svelte';

	interface Props {
		title: string;
		description: string;
		helper?: string;
		/** Notifications and the login button. */
		children: Snippet;
	}

	let { title, description, helper, children }: Props = $props();
</script>

<svelte:head>
	<title>{title}</title>
</svelte:head>

<section>
	<Grid>
		<Column md={6} lg={8}>
			<div class="heading">
				<h1>{title}</h1>
				<p class="description">{description}</p>
			</div>
		</Column>
	</Grid>
	<Grid>
		<Column md={4} lg={4}>
			<div class="form">
				<div class="body">
					{@render children()}
				</div>
				{#if helper}<p class="helper">{helper}</p>{/if}
			</div>
		</Column>
	</Grid>
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
		padding-top: var(--cds-spacing-09);
	}

	.form {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
	}

	.heading {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		margin-bottom: var(--cds-spacing-05);
	}

	h1,
	p {
		margin: 0;
	}

	h1 {
		color: var(--cds-text-primary);
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

	.body {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	.helper {
		color: var(--cds-text-helper);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
