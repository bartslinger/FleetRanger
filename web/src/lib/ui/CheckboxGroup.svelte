<!--
	@component
	Carbon's checkbox group: a fieldset with a legend, a checkbox per option,
	posted under one name, and an optional helper text below.

	Options can have a description under their label, which Carbon's checkbox
	group doesn't have (one helper for the group only): roles need saying what
	they allow. Each is tied to its checkbox for screen readers, as the group's
	helper is to the group; options with one are 16px apart.
-->
<script lang="ts">
	import Checkbox from './Checkbox.svelte';

	interface Props {
		legend: string;
		name: string;
		/** `helper`: what this option means, below its label. A disabled option
		 * isn't posted; the page says why. */
		options: { value: string; label: string; helper?: string; disabled?: boolean }[];
		/** The values checked at first. */
		value: string[];
		helper?: string;
	}

	let { legend, name, options, value, helper }: Props = $props();

	const id = $props.id();
	const described = $derived(options.some((option) => option.helper));
</script>

<fieldset class:described aria-describedby={helper ? `${id}-helper` : undefined}>
	<legend>{legend}</legend>
	{#each options as option, index (option.value)}
		<div class="block">
			<div class="option">
				<Checkbox
					label={option.label}
					{name}
					value={option.value}
					checked={value.includes(option.value)}
					disabled={option.disabled}
					aria-describedby={option.helper ? `${id}-${index}` : undefined}
				/>
			</div>
			{#if option.helper}
				<p class="helper option-helper" id="{id}-{index}">{option.helper}</p>
			{/if}
		</div>
	{/each}
	{#if helper}<p class="helper" id="{id}-helper">{helper}</p>{/if}
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

	fieldset.described {
		gap: var(--cds-spacing-05);
	}

	/* Under the option's label: past the 20px box and its 10px gap. */
	.helper.option-helper {
		padding-inline-start: 1.875rem;
	}

	.helper {
		margin: 0;
		color: var(--cds-text-helper);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
