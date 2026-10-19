<!--
	@component
	Carbon's multiselect, for a filter that takes several values of one
	attribute: a menu of options with checkboxes that stays open while the
	user picks. The field shows a tag with the number chosen, whose × clears
	them, and the chosen names. Chosen options come first when the menu opens.
	The choice applies when the menu closes, not on every checkbox, so a list
	it filters doesn't reload while the user picks. No "select all": for a
	filter, all and none mean the same. Bits UI handles the keyboard
	(arrows, typeahead, Space/Enter, Escape) and focus.
-->
<script lang="ts">
	import { Select } from 'bits-ui';
	import Checkmark from '@carbon/icons/es/checkmark/16.js';
	import ChevronDown from '@carbon/icons/es/chevron--down/16.js';
	import Close from '@carbon/icons/es/close/16.js';
	import Field from './Field.svelte';
	import Icon from './Icon.svelte';

	interface Option {
		value: string;
		label: string;
	}

	interface Props {
		label: string;
		options: Option[];
		/** What the field says with nothing chosen, e.g. "All drones". */
		placeholder: string;
		/** The applied choice. */
		value: string[];
		/** A new choice, when the menu closes or the tag clears it. */
		onchange: (value: string[]) => void;
	}

	let { label, options, placeholder, value, onchange }: Props = $props();

	const id = $props.id();
	let open = $state(false);
	// While the menu is open, the choice being made.
	let picking: string[] = $state([]);
	// The order while the menu is open: chosen first, as they were when it opened.
	let ordered: Option[] = $state([]);

	const chosen = $derived(open ? picking : value);
	const names = $derived(
		options
			.filter((option) => chosen.includes(option.value))
			.map((option) => option.label)
			.join(', ')
	);

	function openChanged(next: boolean) {
		if (next) {
			picking = [...value];
			ordered = [
				...options.filter((option) => value.includes(option.value)),
				...options.filter((option) => !value.includes(option.value))
			];
		} else if (!same(picking, value)) {
			onchange(picking);
		}
		open = next;
	}

	const same = (a: string[], b: string[]) =>
		a.length === b.length && a.every((item) => b.includes(item));

	function clear(event: MouseEvent) {
		event.stopPropagation();
		onchange([]);
	}
</script>

<Field {id} {label}>
	<div class="wrapper">
		<Select.Root
			type="multiple"
			bind:value={picking}
			{open}
			onOpenChange={openChanged}
			items={options}
		>
			<Select.Trigger {id} class="multiselect-field">
				{#if chosen.length > 0}
					<span class="spacer" aria-hidden="true"></span>
				{/if}
				<span class="text" class:placeholder={chosen.length === 0}>
					{chosen.length > 0 ? names : placeholder}
				</span>
				<span class="chevron"><Icon icon={ChevronDown} /></span>
			</Select.Trigger>
			<Select.Portal>
				<Select.Content class="multiselect-menu" sideOffset={0} collisionPadding={16}>
					<Select.Viewport>
						{#each ordered as option (option.value)}
							<Select.Item value={option.value} label={option.label} class="multiselect-option">
								{#snippet children({ selected })}
									<span class="box" class:checked={selected} aria-hidden="true">
										{#if selected}<Icon icon={Checkmark} />{/if}
									</span>
									<span class="option-label">{option.label}</span>
								{/snippet}
							</Select.Item>
						{/each}
					</Select.Viewport>
				</Select.Content>
			</Select.Portal>
		</Select.Root>
		{#if chosen.length > 0}
			<!-- Carbon's filter tag: the count, and × to clear them all. Outside
			     the trigger, as a button can't hold a button. -->
			<button
				type="button"
				class="tag"
				aria-label={`Clear ${chosen.length} chosen ${label.toLowerCase()}`}
				onclick={clear}
			>
				{chosen.length}
				<Icon icon={Close} />
			</button>
		{/if}
	</div>
</Field>

<style>
	.wrapper {
		position: relative;
		display: flex;
	}

	:global(.multiselect-field) {
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

	:global(.multiselect-field:hover) {
		background: var(--cds-field-hover);
	}

	:global(.multiselect-field:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Room for the tag, which lies over the field's start. */
	.spacer {
		flex: none;
		width: 2.75rem;
	}

	.text {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.placeholder {
		color: var(--cds-text-primary);
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

	/* Carbon's high-contrast filter tag: 24px, inverse, with the close icon. */
	.tag {
		position: absolute;
		top: 50%;
		left: var(--cds-spacing-03);
		display: inline-flex;
		align-items: center;
		gap: var(--cds-spacing-01);
		height: 1.5rem;
		padding: 0 var(--cds-spacing-02) 0 var(--cds-spacing-03);
		border: 0;
		border-radius: 1rem;
		background: var(--cds-background-inverse);
		color: var(--cds-text-inverse);
		font: inherit;
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
		transform: translateY(-50%);
		cursor: pointer;
	}

	.tag:hover {
		background: var(--cds-background-inverse-hover);
	}

	.tag:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: 1px;
	}

	/* Carbon's list box: on the layer, with a shadow; options 40px, 48px on
	   phones for touch, each with a checkbox. */
	:global(.multiselect-menu) {
		z-index: 10;
		width: var(--bits-select-anchor-width);
		max-height: min(20rem, var(--bits-select-content-available-height));
		overflow-y: auto;
		background: var(--cds-layer-01);
		box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.3);
	}

	:global(.multiselect-option) {
		display: flex;
		align-items: center;
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
		:global(.multiselect-option) {
			min-height: 3rem;
		}
	}

	:global(.multiselect-option[data-highlighted]) {
		background: var(--cds-layer-hover-01);
		color: var(--cds-text-primary);
	}

	:global(.multiselect-option[data-selected]) {
		color: var(--cds-text-primary);
	}

	/* Carbon's checkbox: 16px, 1px border, filled with a checkmark when chosen. */
	.box {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 1rem;
		height: 1rem;
		border: 1px solid var(--cds-icon-primary);
		border-radius: 2px;
		color: var(--cds-icon-inverse);
	}

	.box.checked {
		background: var(--cds-icon-primary);
	}

	.option-label {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
</style>
