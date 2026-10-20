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
		/** Columns left empty before it from md and from lg, e.g. to centre it. */
		mdOffset?: number;
		lgOffset?: number;
		narrow?: boolean;
		/** Below md only, a later place in the grid, e.g. 1 to come after the
		 * columns without one: facts below a table on phones. */
		smOrder?: number;
		children: Snippet;
	}

	let {
		sm = 4,
		md = 8,
		lg = 16,
		mdOffset,
		lgOffset,
		narrow = false,
		smOrder,
		children
	}: Props = $props();
</script>

<div
	class="column"
	class:narrow
	style:--span-sm={sm}
	style:--span-md={md}
	style:--span-lg={lg}
	style:--start-md={mdOffset === undefined ? undefined : mdOffset + 1}
	style:--start-lg={lgOffset === undefined ? undefined : lgOffset + 1}
	style:--order-sm={smOrder}
>
	{@render children()}
</div>

<style>
	.column {
		--span: var(--span-sm);

		/* Half the gutter on each side: 16px + 16px between columns. */
		grid-column: var(--start, auto) / span var(--span);
		order: var(--order-sm, 0);
		margin-inline: calc(var(--grid-gutter) / 2);
		min-width: 0;
	}

	.narrow {
		margin-inline-start: 0;
	}

	@media (min-width: 42rem) {
		.column {
			--span: var(--span-md);
			--start: var(--start-md, auto);
			order: 0;
		}
	}

	@media (min-width: 66rem) {
		.column {
			--span: var(--span-lg);
			--start: var(--start-lg, auto);
		}
	}
</style>
