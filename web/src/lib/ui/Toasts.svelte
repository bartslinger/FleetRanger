<!--
	@component
	Carbon's low-contrast toast notifications, stacked at the top right below
	the header. Rendered once, by the shell; pages call `toast()`.
-->
<script lang="ts">
	import CheckmarkFilled from '@carbon/icons/es/checkmark--filled/20.js';
	import Close from '@carbon/icons/es/close/20.js';
	import InformationFilled from '@carbon/icons/es/information--filled/20.js';
	import Icon from './Icon.svelte';
	import { dismiss, toasts } from './toasts.svelte.js';

	const icons = { success: CheckmarkFilled, info: InformationFilled };
</script>

<!-- One live region, so screen readers announce each toast as it appears. -->
<div class="toasts" role="status" aria-live="polite">
	{#each toasts as toast (toast.id)}
		<div
			class="toast"
			style:--background="var(--cds-notification-background-{toast.kind})"
			style:--accent="var(--cds-support-{toast.kind})"
		>
			<span class="icon"><Icon icon={icons[toast.kind]} label={toast.kind} /></span>
			<p>{toast.title}</p>
			<button
				class="close"
				type="button"
				aria-label="Close notification"
				onclick={() => dismiss(toast.id)}
			>
				<Icon icon={Close} />
			</button>
		</div>
	{/each}
</div>

<style>
	/* 16px from the right and below the header, as Carbon places toasts. */
	.toasts {
		position: fixed;
		top: calc(var(--cds-size-large) + var(--cds-spacing-05));
		right: var(--cds-spacing-05);
		z-index: 7;
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		width: min(18rem, calc(100% - 2 * var(--cds-spacing-05)));
		pointer-events: none;
	}

	.toast {
		display: flex;
		align-items: flex-start;
		gap: var(--cds-spacing-05);
		min-height: var(--cds-size-large);
		padding-left: calc(var(--cds-spacing-05) - 3px);
		border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
		border-left: 3px solid var(--accent);
		background: var(--background);
		box-shadow: 0 2px 6px 0 rgb(0 0 0 / 0.2);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		pointer-events: auto;
		animation: toast-in 240ms cubic-bezier(0, 0, 0.38, 0.9);
	}

	.icon {
		display: flex;
		padding-top: var(--cds-spacing-04);
		color: var(--accent);
	}

	p {
		flex: 1;
		padding-block: 0.9375rem;
		font-weight: var(--cds-heading-compact-01-font-weight);
	}

	.close {
		display: grid;
		place-items: center;
		flex: none;
		width: var(--cds-size-large);
		height: var(--cds-size-large);
		padding: 0;
		border: 0;
		background: none;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	.close:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	@keyframes toast-in {
		from {
			opacity: 0;
			transform: translateX(1rem);
		}
	}
</style>
