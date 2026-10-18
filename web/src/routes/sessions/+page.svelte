<script lang="ts">
	import { untrack } from 'svelte';
	import { resolve } from '$app/paths';
	import { dateTime, distance, duration } from '#lib/format.js';
	import type { Session } from '#lib/server/backend.js';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import RadioTile from '#lib/ui/RadioTile.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap, { type Track } from '#lib/ui/TrackMap.svelte';
	import ViewSwitcher from '#lib/ui/ViewSwitcher.svelte';
	import type { PageProps, Snapshot } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.sessions)[number];
	type View = 'table' | 'cards';

	/** From the first takeoff to the last landing. */
	const span = (session: Row) =>
		(Date.parse(session.last_landed_at) - Date.parse(session.first_takeoff_at)) / 1000;

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'first_takeoff_at',
			header: `Start (${data.timeZoneName})`,
			format: (session) => dateTime(session.first_takeoff_at, data.timeZone)
		},
		{ key: 'last_landed_at', header: 'Duration', numeric: true, format: (s) => duration(span(s)) },
		{ key: 'flights', header: 'Flights', numeric: true },
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (session) => duration(session.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			format: (session) => distance(session.distance_m)
		},
		{ key: 'pilots', header: 'Pilot', format: (session) => session.pilots.join(', ') },
		{ key: 'teams', header: 'Team', format: (session) => session.teams.join(', ') }
	]);

	// Merging: admins change any session, members those of their own team.
	let selected: string[] = $state([]);
	const canChange = $derived(data.choices.admin || data.choices.own_team_id !== null);
	const changeable = (session: Row) =>
		data.choices.admin || session.team_ids.every((team) => team === data.choices.own_team_id);

	function merged(count: number) {
		selected = [];
		toast(`${count} sessions merged.`);
	}

	/** The list's URL; defaults are left out. */
	function href(page: number, size: number, view: View = data.view) {
		const query = [
			page > 1 ? `page=${page}` : '',
			size !== data.defaultSize ? `size=${size}` : '',
			view !== 'table' ? `view=${view}` : ''
		]
			.filter(Boolean)
			.join('&');
		return `${resolve('/sessions')}${query ? `?${query}` : ''}`;
	}

	// Cards: the map shows the chosen session, the first one until another is chosen.
	let chosen: string | undefined = $state();
	const shown = $derived(
		data.sessions.find((session) => session.id === chosen) ?? data.sessions[0]
	);

	// Below lg the map is above the cards: bring it into view, but only for a
	// tap or click, so arrow keys don't move the focused card out of view.
	let mapSide: HTMLElement | undefined = $state();
	function choose(id: string, pointer: boolean) {
		chosen = id;
		if (pointer && !matchMedia('(min-width: 66rem)').matches) {
			const motion = matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth';
			mapSide?.scrollIntoView({ behavior: motion, block: 'start' });
		}
	}

	// Kept in the browser's history, so going back from a session's page
	// restores the choice and the selection.
	export const snapshot: Snapshot<{ chosen?: string; selected: string[] }> = {
		capture: () => ({ chosen, selected }),
		restore: (value) => ({ chosen, selected } = value)
	};

	// The chosen session's tracks, fetched when it is chosen; not cached, as
	// merging and splitting change them.
	// The first session's tracks came with the page; later choices (and other
	// pages of the list) fetch theirs.
	const initial = untrack(() => data.firstTracks);
	let shownTracks: Track[] = $state(initial?.tracks ?? []);
	let tracksFor: string | undefined = initial?.id;
	let tracksLoading = $state(false);
	let tracksFailed = $state(false);

	$effect(() => {
		if (data.view !== 'cards' || !shown) return;
		const id = shown.id;
		if (id === tracksFor) return;
		tracksFor = id;
		let cancelled = false;
		let finished = false;
		// Never the previous session's tracks under this session's heading;
		// the map keeps its view until the new ones arrive.
		shownTracks = [];
		tracksLoading = true;
		tracksFailed = false;
		fetch(`/api/sessions/${id}`)
			.then((response) => {
				if (!response.ok) throw new Error(String(response.status));
				return response.json() as Promise<Session>;
			})
			.then((session) => {
				if (!cancelled) shownTracks = session.flights.map((flight) => flight.track);
			})
			.catch(() => {
				if (!cancelled) tracksFailed = true;
			})
			.finally(() => {
				finished = true;
				if (!cancelled) tracksLoading = false;
			});
		return () => {
			cancelled = true;
			// Cut off before the tracks came: load them again next time.
			if (!finished) tracksFor = undefined;
		};
	});

	const plural = (count: number, noun: string) => `${count} ${noun}${count === 1 ? '' : 's'}`;
</script>

<PageHeader title="Sessions">
	{#snippet actions()}
		<ViewSwitcher
			label="View"
			options={[
				{
					label: 'Table',
					href: href(data.page, data.size, 'table'),
					selected: data.view === 'table'
				},
				{
					label: 'Cards',
					href: href(data.page, data.size, 'cards'),
					selected: data.view === 'cards'
				}
			]}
		/>
	{/snippet}
</PageHeader>

<Grid rowGap>
	{#if data.total === 0}
		<Column narrow>
			<EmptyState
				title="No sessions yet"
				body="A pilot’s flights are grouped into sessions once the flights have a pilot. Set pilots in the logbook."
			/>
		</Column>
	{:else if data.view === 'cards'}
		<Column narrow>
			<div class="cards-view">
				<fieldset class="cards">
					<legend class="visually-hidden">Session shown on the map</legend>
					{#each data.sessions as session (session.id)}
						<RadioTile
							name="session"
							value={session.id}
							checked={session.id === shown?.id}
							onselect={choose}
						>
							<span class="card-title">{dateTime(session.first_takeoff_at, data.timeZone)}</span>
							<span class="card-who">{session.pilots.join(', ')} · {session.teams.join(', ')}</span>
							<span class="card-numbers">
								{plural(session.flights, 'flight')} · {duration(session.air_time_s)} in the air · {distance(
									session.distance_m
								)}
							</span>
						</RadioTile>
					{/each}
				</fieldset>
				<div class="map-side" bind:this={mapSide}>
					{#if shown}
						<div class="map-heading">
							<h2>{dateTime(shown.first_takeoff_at, data.timeZone)}</h2>
							<a href={resolve('/sessions/[id]', { id: shown.id })}>View session</a>
						</div>
					{/if}
					{#if tracksFailed}
						<InlineNotification kind="error" title="The tracks couldn’t load">
							Try again later.
						</InlineNotification>
					{:else}
						<TrackMap
							tracks={shownTracks}
							token={data.mapboxToken}
							label="Map of the chosen session’s tracks"
							loading={tracksLoading}
							squareOnPhones
						/>
					{/if}
				</div>
			</div>
		</Column>
		<Column narrow>
			<Pagination
				page={data.page}
				size={data.size}
				total={data.total}
				sizes={data.sizes}
				noun="sessions"
				href={(page, size) => href(page, size)}
			/>
		</Column>
	{:else}
		<Column narrow>
			<DataTable
				label="Sessions"
				{columns}
				rows={data.sessions}
				rowKey={(session) => session.id}
				rowHref={(session) => resolve('/sessions/[id]', { id: session.id })}
				selectable={canChange ? changeable : undefined}
				bind:selected
				selectedLabel={(count) => `${count} ${count === 1 ? 'session' : 'sessions'} selected`}
			>
				{#snippet batchActions(ids)}
					{#if ids.length > 1}
						<Modal
							title="Merge sessions"
							submitLabel="Merge"
							action="?/merge"
							error={form?.mergeMessage}
							onsuccess={() => merged(ids.length)}
						>
							{#each ids as id (id)}<input type="hidden" name="session_id" value={id} />{/each}
							<p>
								The flights of the {ids.length} sessions become one session. Split it on the session’s
								page to undo this.
							</p>
						</Modal>
					{/if}
				{/snippet}
			</DataTable>
			<Pagination
				page={data.page}
				size={data.size}
				total={data.total}
				sizes={data.sizes}
				noun="sessions"
				href={(page, size) => href(page, size)}
			/>
		</Column>
	{/if}
</Grid>

<style>
	/* Tiles 2px apart, as in Carbon's tile groups. */
	.cards {
		display: grid;
		gap: var(--cds-spacing-01);
		margin: 0;
		padding: 0;
		border: 0;
	}

	.card-title {
		display: block;
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.card-who,
	.card-numbers {
		display: block;
		margin-top: var(--cds-spacing-02);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.card-numbers {
		color: var(--cds-text-secondary);
		font-variant-numeric: tabular-nums;
	}

	/* The map above the cards on small screens; from lg, 5 of 16 columns of
	   cards and the map beside them, in view while the cards scroll. */
	.cards-view {
		display: grid;
		gap: var(--cds-spacing-07);
	}

	.map-side {
		order: -1;
	}

	/* On the page's 16 columns: narrow columns, so 16px apart. */
	@media (min-width: 66rem) {
		.cards-view {
			grid-template-columns: repeat(16, minmax(0, 1fr));
			column-gap: var(--cds-spacing-05);
			align-items: start;
		}

		.cards {
			grid-column: span 5;
		}

		.map-side {
			position: sticky;
			top: calc(3rem + var(--cds-spacing-05));
			grid-column: span 11;
			order: 0;
		}
	}

	/* The chosen session's date, and the way to its page. */
	.map-heading {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--cds-spacing-05);
		margin-bottom: var(--cds-spacing-03);
		padding-inline: var(--cds-spacing-05);
	}

	.map-heading h2 {
		margin: 0;
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.map-heading a {
		color: var(--cds-link-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
	}

	.map-heading a:hover {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}
</style>
