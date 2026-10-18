<script lang="ts">
	import { resolve } from '$app/paths';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import Select from '#lib/ui/Select.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime, distance, duration } from '#lib/format.js';
	import { DEFAULT_SORT, PAGE_SIZES, query, type LogbookState } from './logbook.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.flights)[number];

	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: `Takeoff (${data.timeZoneName})`,
			sortable: true,
			format: (flight) => dateTime(flight.takeoff_at, data.timeZone)
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			sortable: true,
			format: (flight) => duration(flight.air_time_s)
		},
		{
			key: 'distance_m',
			header: 'Distance',
			numeric: true,
			sortable: true,
			format: (flight) => distance(flight.distance_m)
		},
		{ key: 'pilot_name', header: 'Pilot' },
		{ key: 'team_name', header: 'Team' },
		{ key: 'aircraft_serial', header: 'Drone serial', identifier: true, sortable: true },
		{ key: 'rc_serial', header: 'Controller serial', identifier: true, sortable: true }
	]);

	// Batch actions: admins change any flight, members those of their own team.
	let selected: string[] = $state([]);
	const canChange = $derived(data.choices.admin || data.choices.own_team_id !== null);
	const changeable = (flight: Row) =>
		data.choices.admin || flight.team_id === data.choices.own_team_id;

	const pilotOptions = $derived([
		{ value: '', label: 'Unknown' },
		...data.choices.pilots.map((pilot) => ({
			value: pilot.id,
			label: data.choices.admin ? `${pilot.name} (${pilot.team_name})` : pilot.name
		}))
	]);
	const teamOptions = $derived([
		...data.choices.teams.map((team) => ({ value: team.id, label: team.name })),
		{ value: '', label: 'No team (organisation queue)' }
	]);

	function done(count: number) {
		selected = [];
		toast(`${count} ${count === 1 ? 'flight' : 'flights'} changed.`);
	}

	function href(state: LogbookState) {
		return `${resolve('/')}${query(state)}`;
	}
</script>

<PageHeader title="Logbook" />

<Grid>
	<Column narrow>
		{#if data.total === 0}
			<EmptyState
				title="No flights yet"
				body="Flights appear here when flight logs from your remote controllers are imported."
			/>
		{:else}
			<DataTable
				label="Flights"
				{columns}
				rows={data.flights}
				rowKey={(flight) => flight.id}
				rowHref={(flight) => resolve('/flights/[id]', { id: flight.id })}
				sort={data.state.sort}
				defaultSort={DEFAULT_SORT}
				sortHref={(sort) => href({ ...data.state, page: 1, sort: sort ?? DEFAULT_SORT })}
				selectable={canChange ? changeable : undefined}
				bind:selected
				selectedLabel={(count) => `${count} ${count === 1 ? 'flight' : 'flights'} selected`}
			>
				{#snippet batchActions(ids)}
					<Modal
						title="Set pilot"
						submitLabel="Set pilot"
						action="?/setPilot"
						error={form && 'pilotMessage' in form ? form.pilotMessage : undefined}
						onsuccess={() => done(ids.length)}
					>
						{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
						<Select label="Pilot" name="pilot_id" options={pilotOptions} />
						<p class="help">
							{data.choices.admin
								? 'The flights move to the pilot’s team. With Unknown, they keep their team.'
								: 'Choose Unknown for a flight that wasn’t flown by someone in your team. It stays in your team.'}
						</p>
					</Modal>
					{#if data.choices.admin}
						<Modal
							title="Move to team"
							submitLabel="Move"
							action="?/setTeam"
							error={form && 'teamMessage' in form ? form.teamMessage : undefined}
							onsuccess={() => done(ids.length)}
						>
							{#each ids as id (id)}<input type="hidden" name="flight_id" value={id} />{/each}
							<Select label="Team" name="team_id" options={teamOptions} />
							<p class="help">A pilot who isn’t in the new team is cleared.</p>
						</Modal>
					{/if}
				{/snippet}
			</DataTable>
			<Pagination
				page={data.state.page}
				size={data.state.size}
				total={data.total}
				sizes={PAGE_SIZES}
				noun="flights"
				href={(page, size) => href({ ...data.state, page, size })}
			/>
		{/if}
	</Column>
</Grid>

<style>
	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
