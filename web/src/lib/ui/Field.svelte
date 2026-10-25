<!--
	@component
	Label, helper text and error message around a form control. The control's
	message must be described by `{id}-message`.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		id: string;
		label: string;
		helper?: string;
		error?: string;
		disabled?: boolean | null;
		/** Carbon's character counter at the end of the label row, e.g. "12/4000". */
		counter?: string;
		children: Snippet;
	}

	let { id, label, helper, error, disabled, counter, children }: Props = $props();
</script>

<div class="field" class:disabled>
	{#if counter}
		<div class="label-row">
			<label for={id}>{label}</label>
			<span class="counter">{counter}</span>
		</div>
	{:else}
		<label for={id}>{label}</label>
	{/if}
	{@render children()}
	{#if error}
		<p id="{id}-message" class="error">{error}</p>
	{:else if helper}
		<p id="{id}-message">{helper}</p>
	{/if}
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		min-width: 0;
	}

	.label-row {
		display: flex;
		justify-content: space-between;
		gap: var(--cds-spacing-03);
	}

	label,
	.counter {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	p {
		margin-top: calc(-1 * var(--cds-spacing-02));
		color: var(--cds-text-helper);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}

	.error {
		color: var(--cds-text-error);
	}

	.disabled label,
	.disabled .counter,
	.disabled p {
		color: var(--cds-text-disabled);
	}
</style>
