<!--
	@component
	Carbon's clickable tile: the whole tile is one link, with no links or
	buttons inside it, and an arrow at the bottom right that marks it as
	navigation. `marked` gives it the selected-row background, e.g. for the
	item whose details show elsewhere on the page.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import ArrowRight from '@carbon/icons/es/arrow--right/20.js';
	import Icon from './Icon.svelte';

	interface Props extends HTMLAnchorAttributes {
		href: string;
		marked?: boolean;
		children: Snippet;
	}

	let { href, marked = false, children, ...rest }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller -->
<a class="tile" class:marked {href} {...rest}>
	{@render children()}
	<span class="arrow"><Icon icon={ArrowRight} /></span>
</a>

<style>
	/* Room on the right for the arrow, 20px at 12px from the corner. */
	.tile {
		position: relative;
		display: block;
		min-height: 4rem;
		padding: var(--cds-spacing-05) var(--cds-spacing-08) var(--cds-spacing-05) var(--cds-spacing-05);
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

	.arrow {
		position: absolute;
		right: var(--cds-spacing-04);
		bottom: var(--cds-spacing-04);
		display: flex;
		color: var(--cds-icon-interactive);
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
