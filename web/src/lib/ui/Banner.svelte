<!--
	@component
	A page-wide notification that stays, after the banner in Carbon's
	notification pattern, which has no component for it yet. Built like an
	actionable inline notification (low contrast) without a close button, since
	what it says holds until its action is used. Deviations, ours: it sticks
	below the header instead of scrolling away, so it is always on screen, and
	the action takes no focus, being part of the page rather than an event; for
	the same reason it isn't a live region.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import InformationFilled from '@carbon/icons/es/information--filled/20.js';
	import WarningFilled from '@carbon/icons/es/warning--filled/20.js';
	import Icon from './Icon.svelte';

	interface Props {
		kind?: 'warning' | 'info';
		title: string;
		/** A small ghost button, or a form with one. */
		action: Snippet;
		children?: Snippet;
	}

	let { kind = 'info', title, action, children }: Props = $props();

	const icons = { warning: WarningFilled, info: InformationFilled };
</script>

<div
	class="banner"
	style:--background="var(--cds-notification-background-{kind})"
	style:--accent="var(--cds-support-{kind})"
>
	<span class="icon"><Icon icon={icons[kind]} label={kind} /></span>
	<p>
		<strong>{title}</strong>
		{#if children}{@render children()}{/if}
	</p>
	<div class="action">{@render action()}</div>
</div>

<style>
	.banner {
		position: sticky;
		top: var(--cds-size-large);
		z-index: 2;
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		column-gap: var(--cds-spacing-05);
		padding-left: calc(var(--cds-spacing-05) - 3px);
		border-bottom: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
		border-left: 3px solid var(--accent);
		background: var(--background);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.icon {
		display: flex;
		padding-top: var(--cds-spacing-04);
		color: var(--accent);
	}

	p {
		flex: 1 1 16rem;
		padding: calc(var(--cds-spacing-04) + 1px) 0 var(--cds-spacing-04);
	}

	strong {
		margin-right: var(--cds-spacing-02);
		font-weight: var(--cds-heading-compact-01-font-weight);
	}

	/* Carbon's actionable notification: a small ghost button, below the text
	   and in line with it on phones, beside it from md. */
	.action {
		/* In line with the text: past the 20px icon and the gap. */
		margin: 0 0 var(--cds-spacing-03) calc(1.25rem + var(--cds-spacing-05));
	}

	.action :global(.button.ghost) {
		color: var(--cds-link-primary);
	}

	.action :global(.button.ghost:is(:hover, :active)) {
		background: var(--cds-notification-action-hover);
	}

	/* Carbon's md breakpoint. */
	@media (min-width: 42rem) {
		.action {
			align-self: center;
			margin: var(--cds-spacing-03) 0 var(--cds-spacing-03) auto;
		}
	}
</style>
