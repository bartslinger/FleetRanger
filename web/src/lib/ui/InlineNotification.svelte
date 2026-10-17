<!--
	@component
	Carbon's inline notification (low contrast), shown within the page content.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import CheckmarkFilled from '@carbon/icons/es/checkmark--filled/20.js';
	import ErrorFilled from '@carbon/icons/es/error--filled/20.js';
	import InformationFilled from '@carbon/icons/es/information--filled/20.js';
	import WarningFilled from '@carbon/icons/es/warning--filled/20.js';
	import Icon from './Icon.svelte';

	interface Props {
		kind?: 'error' | 'warning' | 'success' | 'info';
		title: string;
		children?: Snippet;
	}

	let { kind = 'info', title, children }: Props = $props();

	const icons = {
		error: ErrorFilled,
		warning: WarningFilled,
		success: CheckmarkFilled,
		info: InformationFilled
	};
</script>

<div
	class="notification"
	role={kind === 'error' ? 'alert' : 'status'}
	style:--background="var(--cds-notification-background-{kind})"
	style:--accent="var(--cds-support-{kind})"
>
	<span class="icon"><Icon icon={icons[kind]} label={kind} /></span>
	<p>
		<strong>{title}</strong>
		{#if children}{@render children()}{/if}
	</p>
</div>

<style>
	.notification {
		display: flex;
		gap: var(--cds-spacing-05);
		max-width: 50rem;
		min-height: var(--cds-size-large);
		padding: var(--cds-spacing-04) var(--cds-spacing-05) var(--cds-spacing-04)
			calc(var(--cds-spacing-05) - 3px);
		border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
		border-left: 3px solid var(--accent);
		background: var(--background);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.icon {
		display: flex;
		color: var(--accent);
	}

	p {
		padding-top: 1px;
	}

	strong {
		margin-right: var(--cds-spacing-02);
		font-weight: var(--cds-heading-compact-01-font-weight);
	}
</style>
