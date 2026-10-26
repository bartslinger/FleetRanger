<!--
	@component
	Carbon's number input: a whole number with − and + buttons, for small
	values changed a step at a time. The buttons are out of the tab order, as
	Carbon prescribes: the arrow keys step while the input has focus. Typed
	text counts on blur or Enter; text that isn't a whole number within the
	limits shows an error and leaves the value as it was. State the limits in
	the helper text. Read-only as Carbon's: readable and focusable, without
	the field's background, the buttons shown but inactive.
-->
<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import Add from '@carbon/icons/es/add/16.js';
	import Subtract from '@carbon/icons/es/subtract/16.js';
	import WarningFilled from '@carbon/icons/es/warning--filled/16.js';
	import Field from './Field.svelte';
	import Icon from './Icon.svelte';

	interface Props extends Omit<
		HTMLInputAttributes,
		'size' | 'value' | 'min' | 'max' | 'type' | 'onchange'
	> {
		label: string;
		helper?: string;
		/** Error message; marks the input invalid. Typed text that isn't a
		 * valid number shows its own. */
		error?: string;
		size?: 'sm' | 'md' | 'lg';
		value: number;
		min?: number;
		max?: number;
		/** After a change by typing, an arrow key or a button. */
		onchange?: (value: number) => void;
	}

	let {
		label,
		helper,
		error,
		size = 'md',
		value = $bindable(),
		min,
		max,
		onchange,
		readonly,
		disabled,
		...rest
	}: Props = $props();

	const id = $props.id();
	let typedError = $state('');
	const shownError = $derived(error || typedError);
	const inactive = $derived(!!readonly || !!disabled);
	const atMin = $derived(min !== undefined && value <= min);
	const atMax = $derived(max !== undefined && value >= max);

	function limits() {
		if (min !== undefined && max !== undefined) return ` from ${min} to ${max}`;
		if (min !== undefined) return ` of ${min} or more`;
		if (max !== undefined) return ` of ${max} or less`;
		return '';
	}

	function commit(next: number) {
		typedError = '';
		if (next === value) return;
		value = next;
		onchange?.(next);
	}

	function step(by: number) {
		if (inactive || (by < 0 && atMin) || (by > 0 && atMax)) return;
		commit(value + by);
	}

	function typed(event: Event & { currentTarget: HTMLInputElement }) {
		const raw = event.currentTarget.value.trim();
		const next = Number(raw);
		if (
			raw === '' ||
			!Number.isInteger(next) ||
			(min !== undefined && next < min) ||
			(max !== undefined && next > max)
		) {
			typedError = `Enter a whole number${limits()}.`;
			return;
		}
		// Shows "7" for "07", also when the value stays the same.
		event.currentTarget.value = String(next);
		commit(next);
	}
</script>

<Field {id} {label} {helper} error={shownError} {disabled}>
	<div class="wrapper {size}" class:invalid={shownError} class:readonly class:disabled>
		<input
			{id}
			type="text"
			inputmode="numeric"
			role="spinbutton"
			aria-valuenow={value}
			aria-valuemin={min}
			aria-valuemax={max}
			value={String(value)}
			{readonly}
			{disabled}
			aria-invalid={shownError ? true : undefined}
			aria-describedby={shownError || helper ? `${id}-message` : undefined}
			onchange={typed}
			onkeydown={(event) => {
				if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
					event.preventDefault();
					step(event.key === 'ArrowUp' ? 1 : -1);
				}
			}}
			{...rest}
		/>
		{#if shownError}
			<span class="error-icon"><Icon icon={WarningFilled} /></span>
		{/if}
		<div class="controls">
			<button
				type="button"
				tabindex="-1"
				aria-label="Decrease {label}"
				disabled={inactive || atMin}
				onclick={() => step(-1)}
			>
				<Icon icon={Subtract} />
			</button>
			<span class="divider"></span>
			<button
				type="button"
				tabindex="-1"
				aria-label="Increase {label}"
				disabled={inactive || atMax}
				onclick={() => step(1)}
			>
				<Icon icon={Add} />
			</button>
		</div>
	</div>
</Field>

<style>
	.wrapper {
		--height: var(--cds-size-medium);
		position: relative;
		display: flex;
		min-width: 9.375rem;
		height: var(--height);
		border-bottom: 1px solid var(--cds-border-strong);
		background: var(--cds-field);
	}

	.wrapper.sm {
		--height: var(--cds-size-small);
	}

	.wrapper.lg {
		--height: var(--cds-size-large);
	}

	/* One ring around the input and its buttons, drawn above a hovered button;
	   focus wins over the error. */
	.wrapper::after {
		content: '';
		position: absolute;
		inset: 0;
		pointer-events: none;
	}

	.wrapper.invalid::after {
		outline: 2px solid var(--cds-support-error);
		outline-offset: -2px;
	}

	.wrapper:has(input:focus-visible)::after {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	.wrapper.readonly {
		border-bottom-color: var(--cds-border-subtle);
		background: transparent;
	}

	.wrapper.disabled {
		border-bottom-color: transparent;
	}

	input {
		flex: 1;
		min-width: 0;
		padding: 0 var(--cds-spacing-05);
		border: 0;
		border-radius: 0;
		outline: 0;
		background: transparent;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		font-variant-numeric: tabular-nums;
	}

	.invalid input {
		padding-right: var(--cds-spacing-09);
	}

	input:disabled {
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	/* 16px before the buttons, with a divider between them. */
	.error-icon {
		position: absolute;
		top: 50%;
		right: calc(2 * var(--height) + var(--cds-spacing-05));
		display: flex;
		color: var(--cds-support-error);
		transform: translateY(-50%);
	}

	.controls {
		position: relative;
		display: flex;
		align-items: center;
	}

	.invalid .controls::before {
		content: '';
		position: absolute;
		left: 0;
		width: 1px;
		height: var(--cds-spacing-05);
		background: var(--cds-border-subtle);
	}

	button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: var(--height);
		height: 100%;
		border: 0;
		background: transparent;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	button:hover:enabled {
		background: var(--cds-field-hover);
		box-shadow: inset 0 -1px 0 var(--cds-border-strong);
	}

	button:disabled {
		color: var(--cds-icon-disabled);
		cursor: not-allowed;
	}

	.readonly button {
		cursor: default;
	}

	.divider {
		width: 1px;
		height: var(--cds-spacing-05);
		background: var(--cds-border-subtle);
	}

	.disabled .divider {
		background: var(--cds-icon-disabled);
	}

	/* No divider beside a hovered button. */
	.controls:has(button:hover:enabled) .divider {
		background: transparent;
	}
</style>
