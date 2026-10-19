<!--
	@component
	Carbon's checkbox group: a fieldset with a legend, a checkbox per option,
	posted under one name, and an optional helper text below.
-->
<script lang="ts">
	import Checkbox from './Checkbox.svelte';

	interface Props {
		legend: string;
		name: string;
		options: { value: string; label: string }[];
		/** The values checked at first. */
		value: string[];
		helper?: string;
	}

	let { legend, name, options, value, helper }: Props = $props();
</script>

<fieldset>
	<legend>{legend}</legend>
	{#each options as option (option.value)}
		<div class="option">
			<Checkbox
				label={option.label}
				{name}
				value={option.value}
				checked={value.includes(option.value)}
			/>
		</div>
	{/each}
	{#if helper}<p class="helper">{helper}</p>{/if}
</fieldset>

<style>
	fieldset {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		min-width: 0;
		margin: 0;
		padding: 0;
		border: 0;
	}

	legend {
		margin-bottom: var(--cds-spacing-03);
		padding: 0;
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.option {
		display: flex;
		align-items: center;
		min-height: var(--cds-size-xsmall);
	}

	/* Touch-sized rows on phones (our rule; Carbon's spacing is tighter). */
	@media (pointer: coarse) {
		.option {
			min-height: 2.75rem;
		}
	}

	.helper {
		margin: 0;
		color: var(--cds-text-helper);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
