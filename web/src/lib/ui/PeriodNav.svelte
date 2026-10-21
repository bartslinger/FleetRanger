<!--
	@component
	Steps through weeks or months (exploring, for the Sessions cards). Carbon
	has no such component; built from its parts as carbon-expert advised: a
	content switcher for the unit, then previous and next as ghost icon
	buttons around the period's name, as in the date picker's calendar
	header, and "This month" or "This week" while another period shows. All
	links, so the period is in the URL.
-->
<script lang="ts">
	import ChevronLeft from '@carbon/icons/es/chevron--left/16.js';
	import ChevronRight from '@carbon/icons/es/chevron--right/16.js';
	import Button from './Button.svelte';
	import IconButton from './IconButton.svelte';
	import ViewSwitcher from './ViewSwitcher.svelte';

	interface Props {
		unit: 'week' | 'month';
		/** Links to the current week and month. */
		units: { week: string; month: string };
		/** E.g. "October 2026". */
		name: string;
		previous: string;
		next: string;
		/** The current period's link, while another one shows. */
		current: string | null;
	}

	let { unit, units, name, previous, next, current }: Props = $props();
</script>

<div class="period">
	<ViewSwitcher
		label="Period"
		size="md"
		lowContrast
		options={[
			{ label: 'Week', href: units.week, selected: unit === 'week' },
			{ label: 'Month', href: units.month, selected: unit === 'month' }
		]}
	/>
	<div class="steps">
		<IconButton icon={ChevronLeft} label="Previous {unit}" href={previous} size="md" />
		<p class="name" aria-live="polite">{name}</p>
		<IconButton icon={ChevronRight} label="Next {unit}" href={next} size="md" />
	</div>
	{#if current}
		<span class="current">
			<Button kind="ghost" size="md" href={current}>This {unit}</Button>
		</span>
	{/if}
</div>

<style>
	.period {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		align-items: center;
	}

	.steps {
		display: flex;
		align-items: center;
	}

	/* Below md the parts join the filter row, which puts its buttons second:
	   the unit and Filters on one line, the steps on the next
	   (carbon-expert's layout review). */
	@media (max-width: 41.99rem) {
		.period {
			display: contents;
		}

		.period > :global(:first-child) {
			order: 1;
		}

		.steps {
			order: 3;
		}

		.current {
			order: 4;
		}
	}

	/* A ghost button's text lines up 16px after what's before it, as the
	   filter row's buttons do: its own padding is the gap. */
	.current {
		display: flex;
		margin-inline-start: calc(-1 * var(--cds-spacing-05));
	}

	/* Wide enough for the longest name, so the arrows don't move. */
	.name {
		min-width: 8.5rem;
		margin: 0;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-align: center;
		font-variant-numeric: tabular-nums;
	}
</style>
