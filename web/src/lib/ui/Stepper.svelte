<!--
	@component
	Previous and next item in a list, with the position: "3 of 47". Stepping
	replaces the history entry, so Back returns to the list, and keeps focus
	on the button pressed. Carbon has no stepper; these are its ghost icon
	buttons as in pagination.
-->
<script lang="ts">
	import ChevronLeft from '@carbon/icons/es/chevron--left/16.js';
	import ChevronRight from '@carbon/icons/es/chevron--right/16.js';
	import IconButton from './IconButton.svelte';

	interface Props {
		/** E.g. "Flights in this list". */
		label: string;
		previous: string | null;
		next: string | null;
		previousLabel: string;
		nextLabel: string;
		position: number;
		total: number;
	}

	let { label, previous, next, previousLabel, nextLabel, position, total }: Props = $props();
</script>

<nav class="stepper" aria-label={label}>
	<IconButton
		icon={ChevronLeft}
		label={previousLabel}
		size="md"
		href={previous ?? undefined}
		disabled={!previous}
		data-sveltekit-replacestate
		data-sveltekit-reset={false}
	/>
	<span class="position" aria-live="polite">{position} of {total}</span>
	<IconButton
		icon={ChevronRight}
		label={nextLabel}
		size="md"
		href={next ?? undefined}
		disabled={!next}
		data-sveltekit-replacestate
		data-sveltekit-reset={false}
	/>
</nav>

<style>
	.stepper {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
	}

	.position {
		min-width: 5rem;
		color: var(--cds-text-secondary);
		text-align: center;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		font-variant-numeric: tabular-nums;
	}
</style>
