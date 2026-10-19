<!--
	@component
	Previous and next flight in the list the user came from (the breadcrumb
	trail's last step, when that is the flights list), with the same filters
	and order across all its pages. Shown only with such a list.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { trail } from '#lib/trail.svelte.js';
	import Stepper from '#lib/ui/Stepper.svelte';
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
		const query = apiQuery(list, page.data.timeZone);
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

	// In takeoff order, say which way in time the list goes. A suggestion's
	// flights are in takeoff order.
	const direction = $derived(
		list?.suggestions
			? ['older', 'newer']
			: list?.sort.key === 'takeoff_at'
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
	<Stepper
		label="Flights in this list"
		previous={place.previous ? href(place.previous) : null}
		next={place.next ? href(place.next) : null}
		{previousLabel}
		{nextLabel}
		position={place.position}
		total={place.total}
	/>
{/if}
