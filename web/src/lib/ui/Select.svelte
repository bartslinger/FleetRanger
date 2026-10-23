<!--
	@component
	Carbon's select: a styled native select, as in Carbon itself. Native menus
	work best on phones. Use a combobox for long or searchable lists.
-->
<script lang="ts">
	import type { HTMLSelectAttributes } from 'svelte/elements';
	import ChevronDown from '@carbon/icons/es/chevron--down/16.js';
	import WarningFilled from '@carbon/icons/es/warning--filled/16.js';
	import Field from './Field.svelte';
	import Icon from './Icon.svelte';

	interface Option {
		value: string;
		label: string;
	}

	interface Props extends Omit<HTMLSelectAttributes, 'size'> {
		label: string;
		options: Option[];
		helper?: string;
		error?: string;
		size?: 'sm' | 'md' | 'lg';
		value?: string;
		/** A prompt such as "Choose a team" for a choice without a default:
		 * shown in the closed field, never in the list (Carbon's disabled,
		 * hidden, selected option). The field's value is then empty. */
		placeholder?: string;
	}

	let {
		label,
		options,
		helper,
		error,
		placeholder,
		size = 'md',
		value = $bindable(),
		...rest
	}: Props = $props();

	const id = $props.id();
</script>

<Field {id} {label} {helper} {error} disabled={rest.disabled}>
	<div class="wrapper">
		<select
			{id}
			class={size}
			class:invalid={error}
			bind:value
			aria-invalid={error ? true : undefined}
			aria-describedby={error || helper ? `${id}-message` : undefined}
			{...rest}
		>
			{#if placeholder}
				<option value="" disabled hidden selected>{placeholder}</option>
			{/if}
			{#each options as option (option.value)}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		{#if error}
			<span class="error-icon"><Icon icon={WarningFilled} /></span>
		{/if}
		<span class="chevron"><Icon icon={ChevronDown} /></span>
	</div>
</Field>

<style>
	.wrapper {
		position: relative;
		display: flex;
	}

	select {
		width: 100%;
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-09) 0 var(--cds-spacing-05);
		border: 0;
		border-bottom: 1px solid var(--cds-border-strong);
		border-radius: 0;
		background: var(--cds-field);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		appearance: none;
		cursor: pointer;
	}

	select.sm {
		height: var(--cds-size-small);
	}

	select.lg {
		height: var(--cds-size-large);
	}

	select:hover {
		background: var(--cds-field-hover);
	}

	/* Room for the error icon before the chevron, as in Carbon's Sass. */
	select.invalid {
		padding-right: var(--cds-spacing-10);
		outline: 2px solid var(--cds-support-error);
		outline-offset: -2px;
	}

	.error-icon {
		position: absolute;
		top: 50%;
		right: var(--cds-spacing-08);
		display: flex;
		color: var(--cds-support-error);
		transform: translateY(-50%);
		pointer-events: none;
	}

	select:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	select:disabled {
		border-bottom-color: transparent;
		background: var(--cds-field);
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	option {
		background: var(--cds-layer-01);
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

	select:disabled + .chevron {
		color: var(--cds-icon-disabled);
	}
</style>
