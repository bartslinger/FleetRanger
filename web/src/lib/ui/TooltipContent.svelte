<!--
	@component
	Carbon's tooltip with its caret: the floating part of a Bits UI
	`Tooltip.Root`, in a portal, inverse, body-compact-01, kept 16px inside
	the window. Two kinds, as in Carbon's Sass: `icon` for an icon-only
	button (2px × 16px padding, 8 × 4px caret, the box 8px from the button)
	and `default` (4px × 16px, 12 × 6px caret, the box 10px from what it
	points at). Bits UI adds the caret's height to `sideOffset`.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Tooltip } from 'bits-ui';

	interface Props {
		kind?: 'icon' | 'default';
		side?: 'top' | 'bottom';
		/** What it points at, if not the trigger, e.g. a mark inside a larger hover area. */
		anchor?: HTMLElement;
		/** The anchor moves, e.g. with the pointer: placed again every frame. */
		follow?: boolean;
		children: Snippet;
	}

	let { kind = 'default', side = 'bottom', anchor, follow = false, children }: Props = $props();

	const caret = $derived(kind === 'icon' ? { width: 8, height: 4 } : { width: 12, height: 6 });
</script>

<Tooltip.Portal>
	<Tooltip.Content
		class="tooltip-content {kind}{follow ? ' follow' : ''}"
		{side}
		sideOffset={4}
		collisionPadding={16}
		customAnchor={anchor}
		updatePositionStrategy={follow ? 'always' : 'optimized'}
		style={follow ? 'pointer-events: none' : undefined}
	>
		<Tooltip.Arrow class="tooltip-caret" width={caret.width} height={caret.height} />
		{@render children()}
	</Tooltip.Content>
</Tooltip.Portal>

<style>
	/* Global: Bits UI renders the content in a portal. */
	:global(.tooltip-content) {
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

	/* Following the pointer, it lets the pointer through to what's under it,
	   else moving towards it ends the hover it belongs to: its wrapper here,
	   itself by its style (Bits UI sets it inline). */
	:global([data-bits-floating-content-wrapper]:has(> .tooltip-content.follow)) {
		pointer-events: none;
	}

	:global(.tooltip-content.icon) {
		padding-block: var(--cds-spacing-01);
	}

	/* The caret fills with currentColor. */
	:global(.tooltip-caret) {
		color: var(--cds-background-inverse);
	}

	:global(.tooltip-caret svg) {
		display: block;
	}
</style>
