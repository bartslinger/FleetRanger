<!--
	@component
	Carbon's radio tile: a tile that is one option of a group, e.g. one session
	of a list whose details show elsewhere on the page. The whole tile is the
	radio button's label; the selected tile gets an outline and a checkmark.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import CheckmarkFilled from '@carbon/icons/es/checkmark--filled/16.js';
	import Icon from './Icon.svelte';

	interface Props {
		/** The radio group's name. */
		name: string;
		value: string;
		checked: boolean;
		onselect: (value: string) => void;
		children: Snippet;
	}

	let { name, value, checked, onselect, children }: Props = $props();
</script>

<label class="tile" class:selected={checked}>
	<input type="radio" {name} {value} {checked} onchange={() => onselect(value)} />
	<span class="check" aria-hidden="true">
		{#if checked}<Icon icon={CheckmarkFilled} />{/if}
	</span>
	<span class="content">{@render children()}</span>
</label>

<style>
	.tile {
		position: relative;
		display: block;
		min-height: 4rem;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
		outline: 2px solid transparent;
		outline-offset: -2px;
		cursor: pointer;
		transition:
			background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9),
			outline-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.tile:hover {
		background: var(--cds-layer-hover-01);
	}

	.tile.selected {
		outline-color: var(--cds-layer-selected-inverse);
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
	}

	.check {
		position: absolute;
		top: var(--cds-spacing-05);
		right: var(--cds-spacing-05);
		display: flex;
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
