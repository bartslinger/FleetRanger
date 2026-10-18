<!--
	@component
	Carbon's checkbox: a native checkbox with its label; both are click targets.
-->
<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	interface Props extends Omit<HTMLInputAttributes, 'type' | 'checked'> {
		label: string;
		checked?: boolean;
	}

	let { label, checked = $bindable(false), ...rest }: Props = $props();
</script>

<label class:disabled={rest.disabled}>
	<input type="checkbox" bind:checked {...rest} />
	{label}
</label>

<style>
	/* The label starts 30px from the left edge: a 20px box for the control and
	   a 10px gap, as in Carbon. */
	label {
		display: inline-flex;
		align-items: center;
		gap: 0.625rem;
		min-height: 1.25rem;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
	}

	label.disabled {
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	/* A 16px box, 2px corners, inset 2px in its 20px box. */
	input {
		position: relative;
		flex: none;
		width: 1rem;
		height: 1rem;
		margin: 2px;
		border: 1px solid var(--cds-icon-primary);
		border-radius: 2px;
		background: transparent;
		appearance: none;
		cursor: inherit;
	}

	input:checked {
		border-color: var(--cds-icon-primary);
		background: var(--cds-icon-primary);
	}

	/* The check mark: two borders of a rotated box, 1.5px wide. */
	input:checked::before {
		content: '';
		position: absolute;
		top: 2px;
		left: 4.5px;
		width: 5px;
		height: 8px;
		border: solid var(--cds-icon-inverse);
		border-width: 0 1.5px 1.5px 0;
		transform: rotate(45deg);
	}

	input:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: 1px;
	}

	input:disabled {
		border-color: var(--cds-icon-disabled);
	}

	input:disabled:checked {
		background: var(--cds-icon-disabled);
	}
</style>
