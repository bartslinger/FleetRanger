<!--
	@component
	A column in a Grid. Spans every column at each breakpoint unless `sm`, `md` or
	`lg` say otherwise.
	`narrow` hangs the column 16px into the gutter, so text inside a container
	lines up with text outside it: use it for tables and tiles, never for
	labelled fields.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Columns spanned from the sm breakpoint (of 4). */
		sm?: number;
		/** Columns spanned from md (of 8). */
		md?: number;
		/** Columns spanned from lg (of 16). */
		lg?: number;
		narrow?: boolean;
		children: Snippet;
	}

	let { sm = 4, md = 8, lg = 16, narrow = false, children }: Props = $props();
</script>

<div class="column" class:narrow style:--span-sm={sm} style:--span-md={md} style:--span-lg={lg}>
	{@render children()}
</div>

<style>
	.column {
		--span: var(--span-sm);

		/* Half the gutter on each side: 16px + 16px between columns. */
		grid-column: span var(--span);
		margin-inline: calc(var(--grid-gutter) / 2);
		min-width: 0;
	}

	.narrow {
		margin-inline-start: 0;
	}

	@media (min-width: 42rem) {
		.column {
			--span: var(--span-md);
		}
	}

	@media (min-width: 66rem) {
		.column {
			--span: var(--span-lg);
		}
	}
</style>
