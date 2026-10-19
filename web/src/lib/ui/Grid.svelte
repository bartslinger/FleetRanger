<!--
	@component
	Carbon's 2x Grid in the product model: 4, 8 or 16 columns, Carbon's margins,
	left-aligned and at most 1584px wide. Place content in Columns.
	`centered` gives the editorial model, for the website: centred in wider windows.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Space rows of columns by the gutter, e.g. for form fields. */
		rowGap?: boolean;
		centered?: boolean;
		children: Snippet;
	}

	let { rowGap = false, centered = false, children }: Props = $props();
</script>

<div class="grid" class:row-gap={rowGap} class:centered>
	{@render children()}
</div>

<style>
	.grid {
		/* Read by Column for its gutters. */
		--grid-gutter: var(--cds-spacing-07);
		--grid-columns: 4;
		--grid-margin: 0;

		display: grid;
		grid-template-columns: repeat(var(--grid-columns), minmax(0, 1fr));
		max-width: 99rem;
		padding-inline: var(--grid-margin);
	}

	.row-gap {
		row-gap: var(--grid-gutter);
	}

	.centered {
		margin-inline: auto;
	}

	/* Carbon's md, lg and max breakpoints (42, 66 and 99rem). Custom properties
	   can't be used in media queries. */
	@media (min-width: 42rem) {
		.grid {
			--grid-columns: 8;
			--grid-margin: var(--cds-spacing-05);
		}
	}

	@media (min-width: 66rem) {
		.grid {
			--grid-columns: 16;
		}
	}

	@media (min-width: 99rem) {
		.grid {
			--grid-margin: var(--cds-spacing-06);
		}
	}
</style>
