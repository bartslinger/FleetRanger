<!--
	@component
	Previous and next flight in the list the user came from (the breadcrumb
	trail's last step, when that is the flights list), with the same filters
	and order across all its pages, and the position: "3 of 47". Shown only
	with such a list. Stepping replaces the history entry, so Back returns to
	the list, and keeps focus on the button pressed. Carbon has no stepper;
	these are its ghost icon buttons as in pagination.
-->
<script lang="ts">
	import ChevronLeft from '@carbon/icons/es/chevron--left/16.js';
	import ChevronRight from '@carbon/icons/es/chevron--right/16.js';
	import { resolve } from '$app/paths';
	import { trail } from '#lib/trail.svelte.js';
	import IconButton from '#lib/ui/IconButton.svelte';
	import { apiQuery, parse, type LogbookState } from '../logbook.js';

	let { id }: { id: string } = $props();

	interface Neighbours {
		previous: string | null;
		next: string | null;
		position: number;
		total: number;
	}

	/** The list the user came from, if the step before this page is one. */
	const list: LogbookState | null = $derived.by(() => {
		const before = trail.steps.at(-1);
		if (!before) return null;
		const url = new URL(before.href, 'http://list');
		return url.pathname === resolve('/flights') ? parse(url.searchParams) : null;
	});

	let place: Neighbours | null = $state(null);

	$effect(() => {
		if (!list) {
			place = null;
			return;
		}
		const query = apiQuery(list);
		let cancelled = false;
		fetch(`/api/flights/${id}/neighbours?${query}`)
			.then((response) => (response.ok ? (response.json() as Promise<Neighbours>) : null))
			.then((found) => {
				if (!cancelled) place = found;
			})
			.catch(() => {
				// The buttons stay as they were; the list still works.
			});
		return () => {
			cancelled = true;
		};
	});

	// In takeoff order, say which way in time the list goes.
	const direction = $derived(
		list?.sort.key === 'takeoff_at'
			? list.sort.descending
				? ['newer', 'older']
				: ['older', 'newer']
			: null
	);
	const previousLabel = $derived(
		direction ? `Previous flight (${direction[0]})` : 'Previous flight'
	);
	const nextLabel = $derived(direction ? `Next flight (${direction[1]})` : 'Next flight');

	const href = (flight: string) => resolve('/flights/[id]', { id: flight });
</script>

{#if place}
	<nav class="stepper" aria-label="Flights in this list">
		<IconButton
			icon={ChevronLeft}
			label={previousLabel}
			size="md"
			href={place.previous ? href(place.previous) : undefined}
			disabled={!place.previous}
			data-sveltekit-replacestate
			data-sveltekit-reset={false}
		/>
		<span class="position" aria-live="polite">{place.position} of {place.total}</span>
		<IconButton
			icon={ChevronRight}
			label={nextLabel}
			size="md"
			href={place.next ? href(place.next) : undefined}
			disabled={!place.next}
			data-sveltekit-replacestate
			data-sveltekit-reset={false}
		/>
	</nav>
{/if}

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
