<!--
	@component
	Carbon's login pattern, default style: a title, a short description, then
	messages and the login action, with optional helper text below. Place it on
	four grid columns.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

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
	<div class="heading">
		<h1>{title}</h1>
		<p class="description">{description}</p>
	</div>
	<div class="body">
		{@render children()}
	</div>
	{#if helper}<p class="helper">{helper}</p>{/if}
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
		padding-top: var(--cds-spacing-09);
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
