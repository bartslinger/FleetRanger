<!--
	@component
	Carbon's progress bar: a label, the bar and helper text. Small (4px) for
	side panels and cards, big (8px) elsewhere. Finished and error fill the bar
	and add a status icon after the label; the helper text says what happened,
	since the status isn't conveyed to assistive technology otherwise.
-->
<script lang="ts">
	import CheckmarkFilled from '@carbon/icons/es/checkmark--filled/16.js';
	import ErrorFilled from '@carbon/icons/es/error--filled/16.js';
	import Icon from './Icon.svelte';

	interface Props {
		label: string;
		value: number;
		max: number;
		/** E.g. "18 of 30 logs"; required for error, to say what went wrong. */
		helper?: string;
		/** Error is for a process that stopped with an error. */
		status?: 'active' | 'finished' | 'error';
		size?: 'small' | 'big';
	}

	let { label, value, max, helper, status = 'active', size = 'big' }: Props = $props();

	const id = $props.id();
	const percent = $derived(
		status === 'active' ? (max > 0 ? Math.min(100, (value / max) * 100) : 0) : 100
	);
</script>

<div class="progress {status}" class:small={size === 'small'}>
	<div class="top">
		<span class="label" id="{id}-label">{label}</span>
		{#if status === 'finished'}
			<span class="icon"><Icon icon={CheckmarkFilled} /></span>
		{:else if status === 'error'}
			<span class="icon"><Icon icon={ErrorFilled} /></span>
		{/if}
	</div>
	<div
		class="track"
		role="progressbar"
		aria-labelledby="{id}-label"
		aria-describedby={helper ? `${id}-helper` : undefined}
		aria-valuemin={0}
		aria-valuemax={max}
		aria-valuenow={status === 'active' ? value : max}
	>
		<div class="bar" style:width="{percent}%"></div>
	</div>
	<!-- In the page from the start, so screen readers announce its changes. -->
	<span class="helper" id="{id}-helper" aria-live="polite">{helper ?? ''}</span>
</div>

<style>
	.progress {
		display: grid;
		gap: var(--cds-spacing-03);
	}

	.top {
		display: flex;
		justify-content: space-between;
		gap: var(--cds-spacing-03);
	}

	.label {
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.icon {
		display: flex;
		align-items: center;
		flex: none;
	}

	.finished .icon {
		color: var(--cds-support-success);
	}

	.error .icon {
		color: var(--cds-support-error);
	}

	.track {
		height: var(--cds-spacing-03);
		background: var(--cds-border-subtle);
	}

	.small .track {
		height: var(--cds-spacing-02);
	}

	.bar {
		height: 100%;
		background: var(--cds-border-interactive);
		transition: width 240ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.finished .bar {
		background: var(--cds-support-success);
	}

	.error .bar {
		background: var(--cds-support-error);
	}

	.helper {
		color: var(--cds-text-helper);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}

	.helper:empty {
		display: none;
	}

	.error .helper {
		color: var(--cds-text-error);
	}

	@media (prefers-reduced-motion: reduce) {
		.bar {
			transition: none;
		}
	}
</style>
