<!--
	@component
	Carbon's clickable tile: the whole tile is one link, with no links or
	buttons inside it. `marked` gives it the selected-row background, e.g. for
	the item whose details show elsewhere on the page.
	Carbon puts an arrow at the bottom right; left out, because these tiles
	come in lists, where every item would repeat it (the user's decision).
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	interface Props extends HTMLAnchorAttributes {
		href: string;
		marked?: boolean;
		children: Snippet;
	}

	let { href, marked = false, children, ...rest }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller -->
<a class="tile" class:marked {href} {...rest}>{@render children()}</a>

<style>
	.tile {
		display: block;
		min-height: 4rem;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
		color: var(--cds-text-primary);
		text-decoration: none;
		transition: background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.tile.marked {
		background: var(--cds-layer-selected-01);
	}

	.tile:hover {
		background: var(--cds-layer-hover-01);
	}

	.tile.marked:hover {
		background: var(--cds-layer-selected-hover-01);
	}

	.tile:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	@media (prefers-reduced-motion: reduce) {
		.tile {
			transition: none;
		}
	}
</style>
