<!--
	@component
	Carbon's small toggle, for an on/off setting that applies at once: a native
	checkbox with the switch role, drawn as Carbon's 32×16 switch with the tick
	in its knob when on. Both the switch and the label are click targets.

	Deviations, for a table toolbar (carbon-expert's advice): the label sits
	beside the switch rather than above it, as there is no room above in 48px;
	the small toggle's optional state text ("On"/"Off") is left out; the focus
	ring shows on keyboard focus only (`:focus-visible`).
-->
<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	interface Props extends Omit<HTMLInputAttributes, 'type' | 'checked' | 'role'> {
		/** Stays the same in both states. */
		label: string;
		checked?: boolean;
	}

	let { label, checked = $bindable(false), ...rest }: Props = $props();
</script>

<label class:disabled={rest.disabled}>
	<input class="visually-hidden" type="checkbox" role="switch" bind:checked {...rest} />
	<span class="switch" aria-hidden="true">
		<svg class="check" viewBox="0 0 6 5"><path d="M2.2 2.7L5 0 6 1 2.2 5 0 2.7 1 1.5z" /></svg>
	</span>
	<span>{label}</span>
</label>

<style>
	label {
		display: inline-flex;
		align-items: center;
		gap: var(--cds-spacing-03);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
	}

	.switch {
		position: relative;
		flex: none;
		width: 2rem;
		height: 1rem;
		border-radius: 0.75rem;
		background: var(--cds-toggle-off);
		transition: background-color 70ms cubic-bezier(0.2, 0, 1, 0.9);
	}

	/* The knob: 10px, 3px from the edges. */
	.switch::before {
		position: absolute;
		top: 3px;
		left: 3px;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--cds-icon-on-color);
		content: '';
		transition: transform 70ms cubic-bezier(0.2, 0, 1, 0.9);
	}

	.check {
		position: absolute;
		top: 6px;
		right: 5px;
		width: 6px;
		height: 5px;
		fill: var(--cds-support-success);
		visibility: hidden;
	}

	input:checked + .switch {
		background: var(--cds-support-success);
	}

	input:checked + .switch::before {
		transform: translateX(1rem);
	}

	input:checked + .switch .check {
		visibility: visible;
	}

	input:focus-visible + .switch {
		outline: 2px solid var(--cds-focus);
		outline-offset: 1px;
	}

	.disabled {
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	input:disabled + .switch {
		background: var(--cds-button-disabled);
	}

	input:disabled + .switch::before {
		background: var(--cds-icon-on-color-disabled);
	}

	input:disabled + .switch .check {
		fill: var(--cds-button-disabled);
	}

	@media (prefers-reduced-motion: reduce) {
		.switch,
		.switch::before {
			transition: none;
		}
	}

	@media (forced-colors: active) {
		input:checked + .switch {
			background: Highlight;
		}
	}
</style>
