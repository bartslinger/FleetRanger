<!--
	@component
	Carbon's radio tile: a tile that is one option of a group, e.g. one session
	of a list whose details show elsewhere on the page. The whole tile is the
	radio button's label. An unselected tile shows an empty radio icon, so it
	reads as selectable; the selected one a checkmark and a 1px border. The
	2px outline is for focus only.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import CheckmarkFilled from '@carbon/icons/es/checkmark--filled/16.js';
	import RadioButton from '@carbon/icons/es/radio-button/16.js';
	import Icon from './Icon.svelte';

	interface Props {
		/** The radio group's name. */
		name: string;
		value: string;
		checked: boolean;
		/** `pointer`: chosen by tap or click, not with the arrow keys. */
		onselect: (value: string, pointer: boolean) => void;
		children: Snippet;
	}

	let { name, value, checked, onselect, children }: Props = $props();

	// Arrow keys also select radios; a pointer press comes just before.
	let pressed = false;
</script>

<label class="tile" class:selected={checked} onpointerdown={() => (pressed = true)}>
	<input
		type="radio"
		{name}
		{value}
		{checked}
		onchange={() => {
			onselect(value, pressed);
			pressed = false;
		}}
	/>
	<span class="check" class:checked aria-hidden="true">
		<Icon icon={checked ? CheckmarkFilled : RadioButton} />
	</span>
	<span class="content">{@render children()}</span>
</label>

<style>
	.tile {
		position: relative;
		display: block;
		min-height: 4rem;
		padding: var(--cds-spacing-05);
		border: 1px solid transparent;
		background: var(--cds-layer-01);
		cursor: pointer;
		transition:
			background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9),
			border-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.tile:hover {
		background: var(--cds-layer-hover-01);
	}

	.tile.selected {
		border-color: var(--cds-border-inverse);
	}

	/* The input stays focusable and in the accessibility tree. */
	input {
		position: absolute;
		width: 1px;
		height: 1px;
		margin: -1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}

	.tile:has(input:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	.check {
		position: absolute;
		top: var(--cds-spacing-05);
		right: var(--cds-spacing-05);
		display: flex;
		color: var(--cds-icon-secondary);
	}

	.check.checked {
		color: var(--cds-icon-primary);
	}

	.content {
		display: block;
		padding-right: var(--cds-spacing-07);
	}

	@media (prefers-reduced-motion: reduce) {
		.tile {
			transition: none;
		}
	}
</style>
