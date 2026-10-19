<!--
	@component
	Previous and next session in the Sessions list, newest first: with the
	filters of the list the user came from (the breadcrumb trail's last step,
	when that is the Sessions list), else the whole list.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { trail } from '#lib/trail.svelte.js';
	import Stepper from '#lib/ui/Stepper.svelte';
	import { apiFilters, parse } from '../list.js';

	let { id }: { id: string } = $props();

	interface Neighbours {
		previous: string | null;
		next: string | null;
		position: number;
		total: number;
	}

	/** The list's filters as the backend takes them. */
	const filters = $derived.by(() => {
		const before = trail.steps.at(-1);
		const url = new URL(before?.href ?? '/', 'http://list');
		const params = url.pathname === resolve('/sessions') ? url.searchParams : new URLSearchParams();
		return apiFilters(parse(params), page.data.timeZone).toString();
	});

	let place: Neighbours | null = $state(null);

	$effect(() => {
		let cancelled = false;
		fetch(`/api/sessions/${id}/neighbours?${filters}`)
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

	const href = (session: string) => resolve('/sessions/[id]', { id: session });
</script>

{#if place}
	<Stepper
		label="Sessions"
		previous={place.previous ? href(place.previous) : null}
		next={place.next ? href(place.next) : null}
		previousLabel="Previous session (newer)"
		nextLabel="Next session (older)"
		position={place.position}
		total={place.total}
	/>
{/if}
