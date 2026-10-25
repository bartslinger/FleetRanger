<!--
	@component
	Carbon's text area, for longer text such as a comment: a label, helper
	text and an error message, and with `limit` Carbon's character counter
	beside the label. Text over the limit stays, so pasting never loses any:
	the field is invalid with `overLimit` as its message until it's shortened
	(carbon-expert's review). Use `characters` to check it.
-->
<script lang="ts" module>
	/** Characters as the backend counts them (Unicode code points). */
	export const characters = (text: string) => [...text].length;
</script>

<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';
	import WarningFilled from '@carbon/icons/es/warning--filled/16.js';
	import Field from './Field.svelte';
	import Icon from './Icon.svelte';

	interface Props extends HTMLTextareaAttributes {
		label: string;
		helper?: string;
		/** Error message; marks the text area invalid. */
		error?: string;
		value?: string;
		/** The text area, e.g. to focus it. */
		ref?: HTMLTextAreaElement | null;
		/** The most characters allowed, shown in the counter. */
		limit?: number;
		/** The message while the text is over the limit. */
		overLimit?: string;
	}

	let {
		label,
		helper,
		error,
		value = $bindable(''),
		ref = $bindable(null),
		rows = 4,
		limit,
		overLimit,
		...rest
	}: Props = $props();

	const id = $props.id();
	const count = $derived(characters(value));
	const over = $derived(limit != null && count > limit);
	const counter = $derived(limit == null ? undefined : `${count}/${limit}`);
	const message = $derived(error ?? (over ? overLimit : undefined));
</script>

<Field {id} {label} {helper} error={message} {counter} disabled={rest.disabled}>
	<div class="wrapper">
		<textarea
			{id}
			{rows}
			class:invalid={message}
			bind:value
			bind:this={ref}
			aria-invalid={message ? true : undefined}
			aria-describedby={message || helper ? `${id}-message` : undefined}
			{...rest}></textarea>
		{#if message}
			<span class="error-icon"><Icon icon={WarningFilled} /></span>
		{/if}
	</div>
</Field>

<style>
	.wrapper {
		position: relative;
		display: flex;
	}

	/* Carbon: 11px above and below, 16px at the sides, body-compact-01. */
	textarea {
		width: 100%;
		min-height: var(--cds-size-medium);
		padding: calc(var(--cds-spacing-04) - 1px) var(--cds-spacing-05);
		border: 0;
		border-bottom: 1px solid var(--cds-border-strong);
		border-radius: 0;
		background: var(--cds-field);
		color: var(--cds-text-primary);
		font-family: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		resize: vertical;
	}

	textarea::placeholder {
		color: var(--cds-text-placeholder);
		opacity: 1;
	}

	textarea:hover {
		background: var(--cds-field-hover);
	}

	textarea.invalid {
		padding-right: var(--cds-spacing-08);
		outline: 2px solid var(--cds-support-error);
		outline-offset: -2px;
	}

	textarea:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	textarea:disabled {
		border-bottom-color: transparent;
		background: var(--cds-field);
		color: var(--cds-text-disabled);
		cursor: not-allowed;
		resize: none;
	}

	.error-icon {
		position: absolute;
		top: var(--cds-spacing-04);
		right: var(--cds-spacing-05);
		display: flex;
		color: var(--cds-support-error);
	}
</style>
