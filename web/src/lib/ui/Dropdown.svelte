<!--
	@component
	Carbon's dropdown: one choice from a list that takes effect at once, such
	as switching organisation (Carbon: for "taking an action, filtering or
	sorting"; a choice a form submits is a Select). The list is Carbon's, not
	the system's: arrow keys move through it and only Enter, Space or a click
	chooses, so nothing happens while a keyboard user looks (user's choice,
	2026-11-12, over the native select there). Bits UI handles the keyboard
	(arrows, typeahead, Escape) and focus. The menu opens in place, not at the
	end of the page, so it takes the theme and layer of what it sits in, such
	as a header panel, and focus stays inside that.
-->
<script lang="ts">
	import { Select } from 'bits-ui';
	import Checkmark from '@carbon/icons/es/checkmark/16.js';
	import ChevronDown from '@carbon/icons/es/chevron--down/16.js';
	import WarningFilled from '@carbon/icons/es/warning--filled/16.js';
	import Field from './Field.svelte';
	import Icon from './Icon.svelte';

	interface Option {
		value: string;
		label: string;
	}

	interface Props {
		label: string;
		options: Option[];
		/** The current choice. */
		value: string;
		/** A choice other than the current one. */
		onchange: (value: string) => void;
		/** Why the last choice failed; Carbon's invalid state. */
		error?: string;
		disabled?: boolean;
	}

	let { label, options, value, onchange, error, disabled = false }: Props = $props();

	const id = $props.id();
	const current = $derived(options.find((option) => option.value === value)?.label ?? '');

	function chosen(next: string) {
		if (next && next !== value) onchange(next);
	}
</script>

<Field {id} {label} {error} {disabled}>
	<div class="wrapper">
		<Select.Root type="single" {value} onValueChange={chosen} items={options} {disabled}>
			<Select.Trigger
				{id}
				class="dropdown-field"
				aria-invalid={error ? 'true' : undefined}
				aria-describedby={error ? `${id}-message` : undefined}
			>
				<span class="text">{current}</span>
				{#if error}<span class="invalid"><Icon icon={WarningFilled} /></span>{/if}
				<span class="chevron"><Icon icon={ChevronDown} /></span>
			</Select.Trigger>
			<Select.Portal disabled>
				<Select.Content class="dropdown-menu" sideOffset={0} collisionPadding={16}>
					<Select.Viewport>
						{#each options as option (option.value)}
							<Select.Item value={option.value} label={option.label} class="dropdown-option">
								{#snippet children({ selected })}
									<span class="option-label">{option.label}</span>
									{#if selected}<Icon icon={Checkmark} />{/if}
								{/snippet}
							</Select.Item>
						{/each}
					</Select.Viewport>
				</Select.Content>
			</Select.Portal>
		</Select.Root>
	</div>
</Field>

<style>
	.wrapper {
		position: relative;
		display: flex;
	}

	/* Carbon's field: md, 40px, on the contextual field colour. */
	:global(.dropdown-field) {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
		width: 100%;
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-09) 0 var(--cds-spacing-05);
		border: 0;
		border-bottom: 1px solid var(--cds-border-strong);
		border-radius: 0;
		background: var(--cds-field);
		color: var(--cds-text-primary);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-align: left;
		cursor: pointer;
	}

	:global(.dropdown-field:hover) {
		background: var(--cds-field-hover);
	}

	:global(.dropdown-field:focus-visible),
	:global(.dropdown-field[aria-invalid='true']) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	:global(.dropdown-field[aria-invalid='true']) {
		outline-color: var(--cds-support-error);
	}

	:global(.dropdown-field:disabled) {
		border-bottom-color: transparent;
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	.text {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.invalid {
		position: absolute;
		top: 50%;
		right: var(--cds-spacing-08);
		display: flex;
		color: var(--cds-support-error);
		transform: translateY(-50%);
	}

	.chevron {
		position: absolute;
		top: 50%;
		right: var(--cds-spacing-05);
		display: flex;
		color: var(--cds-icon-primary);
		transform: translateY(-50%);
		pointer-events: none;
	}

	/* Up while the menu is open, as Carbon's. */
	:global(.dropdown-field[data-state='open']) .chevron {
		transform: translateY(-50%) rotate(180deg);
	}

	/* Carbon's list box: one layer above what it sits on, with a shadow;
	   options 40px, 48px on phones for touch; the chosen one selected, with
	   the checkmark at its end. */
	:global(.dropdown-menu) {
		z-index: 10;
		width: var(--bits-select-anchor-width);
		max-height: min(20rem, var(--bits-select-content-available-height));
		overflow-y: auto;
		background: var(--cds-layer);
		box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.3);
	}

	:global(.dropdown-option) {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--cds-spacing-03);
		min-height: 2.5rem;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
		outline: none;
	}

	@media (max-width: 41.99rem) {
		:global(.dropdown-option) {
			min-height: 3rem;
		}
	}

	:global(.dropdown-option[data-highlighted]) {
		background: var(--cds-layer-hover);
		color: var(--cds-text-primary);
	}

	:global(.dropdown-option[data-selected]) {
		background: var(--cds-layer-selected);
		color: var(--cds-text-primary);
	}

	:global(.dropdown-option[data-selected][data-highlighted]) {
		background: var(--cds-layer-selected-hover);
	}

	.option-label {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
</style>
