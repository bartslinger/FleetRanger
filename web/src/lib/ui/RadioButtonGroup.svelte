<!--
	@component
	Carbon's radio button group: a fieldset with a legend and native radio
	buttons, stacked vertically. Native radios give one tab stop and arrow-key
	selection for free.
-->
<script lang="ts" generics="V extends string">
	interface Option {
		value: V;
		label: string;
	}

	interface Props {
		legend: string;
		/** Shared by the radios; unique on the page. */
		name: string;
		options: Option[];
		value: V;
		onchange?: (value: V) => void;
	}

	let { legend, name, options, value = $bindable(), onchange }: Props = $props();
</script>

<fieldset>
	<legend>{legend}</legend>
	{#each options as option (option.value)}
		<label>
			<input
				type="radio"
				{name}
				value={option.value}
				checked={value === option.value}
				onchange={() => {
					value = option.value;
					onchange?.(option.value);
				}}
			/>
			{option.label}
		</label>
	{/each}
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

	label {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
		min-height: var(--cds-size-xsmall);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
	}

	/* Carbon's radio: an 18px ring with an 8px dot when checked, in a 20px box. */
	input {
		display: grid;
		place-items: center;
		width: 1.125rem;
		height: 1.125rem;
		margin: 1px;
		border: 1px solid var(--cds-icon-primary);
		border-radius: 50%;
		background: transparent;
		appearance: none;
		cursor: pointer;
	}

	input::before {
		content: '';
		width: var(--cds-spacing-03);
		height: var(--cds-spacing-03);
		border-radius: 50%;
		background: var(--cds-icon-primary);
		transform: scale(0);
	}

	input:checked::before {
		transform: scale(1);
	}

	input:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: 1px;
	}

	/* Touch-sized rows on phones (our rule; Carbon's spacing is tighter). */
	@media (pointer: coarse) {
		label {
			min-height: 2.75rem;
		}
	}
</style>
