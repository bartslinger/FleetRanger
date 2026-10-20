<!--
	@component
	A heading with a list of what something can or can't do, each line after
	a check mark or a cross. Not checkboxes: Carbon keeps controls for what
	can be changed, and these can't. The icons are decorative; the heading
	says what the list means.
-->
<script lang="ts">
	import Checkmark from '@carbon/icons/es/checkmark/16.js';
	import Close from '@carbon/icons/es/close/16.js';
	import Icon from './Icon.svelte';

	interface Props {
		heading: string;
		items: string[];
		/** What it can do (check marks), or else what it can't (crosses). */
		granted: boolean;
	}

	let { heading, items, granted }: Props = $props();
</script>

<div class="permissions">
	<h2>{heading}</h2>
	<ul class:granted>
		{#each items as item (item)}
			<li><Icon icon={granted ? Checkmark : Close} />{item}</li>
		{/each}
	</ul>
</div>

<style>
	.permissions {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-04);
	}

	h2,
	ul {
		margin: 0;
	}

	/* A quiet heading: the label style, so the lists don't shout. */
	h2 {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		font-weight: var(--cds-label-01-font-weight);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	ul {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		padding: 0;
		list-style: none;
	}

	/* The 16px icon on the first 20px line, 8px before the text. */
	li {
		display: grid;
		grid-template-columns: 1rem 1fr;
		gap: var(--cds-spacing-03);
		align-items: start;
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	li :global(svg) {
		margin-top: 0.125rem;
		color: var(--cds-icon-secondary);
	}

	.granted li :global(svg) {
		color: var(--cds-support-success);
	}
</style>
