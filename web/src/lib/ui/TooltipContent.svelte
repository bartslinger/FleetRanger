<!--
	@component
	Carbon's tooltip: the floating part of a Bits UI `Tooltip.Root`, in a
	portal, inverse, body-compact-01, kept 16px inside the window. With
	`arrow`, Carbon's caret tip (12 × 6px), with the box 10px from the
	trigger as Carbon's popover caret; Bits UI adds the arrow's height to
	`sideOffset`.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Tooltip } from 'bits-ui';

	interface Props {
		side?: 'top' | 'bottom';
		/** What it points at, if not the trigger, e.g. a mark inside a larger hover area. */
		anchor?: HTMLElement;
		arrow?: boolean;
		children: Snippet;
	}

	let { side = 'bottom', anchor, arrow = false, children }: Props = $props();
</script>

<Tooltip.Portal>
	<Tooltip.Content
		class="tooltip"
		{side}
		sideOffset={4}
		collisionPadding={16}
		customAnchor={anchor}
	>
		{#if arrow}<Tooltip.Arrow class="tooltip-arrow" width={12} height={6} />{/if}
		{@render children()}
	</Tooltip.Content>
</Tooltip.Portal>

<style>
	/* Global: Bits UI renders the content in a portal. */
	:global(.tooltip) {
		z-index: 10;
		max-width: 18rem;
		padding: var(--cds-spacing-02) var(--cds-spacing-05);
		background: var(--cds-background-inverse);
		color: var(--cds-text-inverse);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		border-radius: 2px;
	}

	/* The arrow fills with currentColor. */
	:global(.tooltip-arrow) {
		color: var(--cds-background-inverse);
	}

	:global(.tooltip-arrow svg) {
		display: block;
	}
</style>
