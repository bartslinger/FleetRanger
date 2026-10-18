<!--
	@component
	Carbon's progress bar (big, 8px): a label, the bar and helper text. Meant for
	a container on layer-01, such as a header panel; the track is one layer up.
-->
<script lang="ts">
	interface Props {
		label: string;
		value: number;
		max: number;
		/** E.g. "18 of 30 logs". */
		helper?: string;
		/** Error turns the bar red, for a process that stopped with an error. */
		status?: 'active' | 'error';
	}

	let { label, value, max, helper, status = 'active' }: Props = $props();

	const id = $props.id();
	const percent = $derived(max > 0 ? Math.min(100, (value / max) * 100) : 0);
</script>

<div class="progress" class:error={status === 'error'}>
	<span class="label" id="{id}-label">{label}</span>
	<div
		class="track"
		role="progressbar"
		aria-labelledby="{id}-label"
		aria-describedby={helper ? `${id}-helper` : undefined}
		aria-valuemin={0}
		aria-valuemax={max}
		aria-valuenow={value}
	>
		<div class="bar" style:width="{percent}%"></div>
	</div>
	{#if helper}<span class="helper" id="{id}-helper">{helper}</span>{/if}
</div>

<style>
	.progress {
		display: grid;
		gap: var(--cds-spacing-03);
	}

	.label {
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.track {
		height: var(--cds-spacing-03);
		background: var(--cds-layer-02);
	}

	.bar {
		height: 100%;
		background: var(--cds-interactive);
		transition: width 240ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.error .bar {
		background: var(--cds-support-error);
	}

	.helper {
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}

	@media (prefers-reduced-motion: reduce) {
		.bar {
			transition: none;
		}
	}
</style>
