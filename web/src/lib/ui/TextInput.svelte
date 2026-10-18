<!--
	@component
	Carbon's text input with a label, helper text and an error message.
-->
<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import WarningFilled from '@carbon/icons/es/warning--filled/16.js';
	import Field from './Field.svelte';
	import Icon from './Icon.svelte';

	interface Props extends Omit<HTMLInputAttributes, 'size'> {
		label: string;
		helper?: string;
		/** Error message; marks the input invalid. */
		error?: string;
		size?: 'sm' | 'md' | 'lg';
		value?: string;
	}

	let {
		label,
		helper,
		error,
		size = 'md',
		value = $bindable(''),
		type = 'text',
		...rest
	}: Props = $props();

	const id = $props.id();
</script>

<Field {id} {label} {helper} {error} disabled={rest.disabled}>
	<div class="wrapper">
		<input
			{id}
			{type}
			class={size}
			class:invalid={error}
			bind:value
			aria-invalid={error ? true : undefined}
			aria-describedby={error || helper ? `${id}-message` : undefined}
			{...rest}
		/>
		{#if error}
			<span class="error-icon"><Icon icon={WarningFilled} /></span>
		{/if}
	</div>
</Field>

<style>
	.wrapper {
		position: relative;
		display: flex;
	}

	input {
		width: 100%;
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-05);
		border: 0;
		border-bottom: 1px solid var(--cds-border-strong);
		border-radius: 0;
		background: var(--cds-field);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	input.sm {
		height: var(--cds-size-small);
	}

	input.lg {
		height: var(--cds-size-large);
	}

	input::placeholder {
		color: var(--cds-text-placeholder);
		opacity: 1;
	}

	input:hover {
		background: var(--cds-field-hover);
	}

	input.invalid {
		padding-right: var(--cds-spacing-08);
		outline: 2px solid var(--cds-support-error);
		outline-offset: -2px;
	}

	input:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	input:disabled {
		border-bottom-color: transparent;
		background: var(--cds-field);
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	.error-icon {
		position: absolute;
		top: 50%;
		right: var(--cds-spacing-05);
		display: flex;
		color: var(--cds-support-error);
		transform: translateY(-50%);
	}
</style>
