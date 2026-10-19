<!--
	@component
	The session to add flights to, for the "Add to session" dialog: the
	sessions within half a day of the flights, fetched when the dialog opens
	(it renders its content only while open).
-->
<script lang="ts">
	import { daySpan } from '#lib/format.js';
	import type { SessionPage, SessionSummary } from '#lib/server/backend.js';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Loading from '#lib/ui/Loading.svelte';
	import Select from '#lib/ui/Select.svelte';

	interface Props {
		/** The flights to add: their takeoff and landing. */
		flights: { takeoff_at: string; landed_at: string }[];
		timeZone: string;
	}

	let { flights, timeZone }: Props = $props();

	/** Sessions this far before the first flight or after the last are offered. */
	const MARGIN_MS = 12 * 60 * 60 * 1000;

	let sessions = $state<SessionSummary[] | null>(null);
	let failed = $state(false);

	$effect(() => {
		if (flights.length === 0) {
			sessions = [];
			return;
		}
		const from = Math.min(...flights.map((flight) => Date.parse(flight.takeoff_at))) - MARGIN_MS;
		const to = Math.max(...flights.map((flight) => Date.parse(flight.landed_at))) + MARGIN_MS;
		const query = new URLSearchParams({
			span_from: new Date(from).toISOString(),
			span_to: new Date(to).toISOString(),
			size: '100'
		});
		let cancelled = false;
		fetch(`/api/sessions?${query}`)
			.then((response) => {
				if (!response.ok) throw new Error(String(response.status));
				return response.json() as Promise<SessionPage>;
			})
			.then((page) => {
				if (!cancelled) sessions = page.sessions;
			})
			.catch(() => {
				if (!cancelled) failed = true;
			});
		return () => {
			cancelled = true;
		};
	});

	const options = $derived(
		(sessions ?? []).map((session) => ({
			value: session.id,
			label: [
				daySpan(session.first_takeoff_at, session.last_landed_at, timeZone),
				session.pilots.join(', ') || 'Unknown pilot',
				`${session.flights} ${session.flights === 1 ? 'flight' : 'flights'}`
			].join(' · ')
		}))
	);
</script>

{#if failed}
	<InlineNotification kind="error" title="The sessions couldn’t load">
		Close this and try again.
	</InlineNotification>
{:else if sessions === null}
	<p class="loading" role="status"><Loading /> Loading sessions…</p>
{:else if sessions.length === 0}
	<p class="none">
		There are no sessions within 12 hours of these flights. Use New session instead.
	</p>
{:else}
	<Select label="Session" name="session_id" {options} />
{/if}

<style>
	.loading {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}

	.loading,
	.none {
		margin: 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}
</style>
