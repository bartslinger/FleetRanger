<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import { clock, dateTime, dayMonth } from '#lib/format.js';
	import { may } from '#lib/permissions.js';
	import Column from '#lib/ui/Column.svelte';
	import FlightFacts from '#lib/FlightFacts.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import FlightStepper from './FlightStepper.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import TrackMap from '#lib/ui/TrackMap.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const flight = $derived(data.flight);
	const archived = $derived(flight.archived_at !== null);
	const canArchive = $derived(may(data.account, 'flights.archive', flight.group_id));
</script>

<PageHeader
	title={`Flight ${dateTime(flight.takeoff_at, data.timeZone)}`}
	description={`${flight.aircraft_name ?? flight.aircraft_serial ?? 'Unknown drone'} · ${flight.pilot_name ?? 'Unknown pilot'}`}
	crumb={`Flight ${dayMonth(flight.takeoff_at, data.timeZone)}, ${clock(flight.takeoff_at, data.timeZone)}`}
	breadcrumb={[{ href: resolve('/flights'), label: 'Flights' }]}
	status={archived ? 'Archived' : undefined}
>
	{#snippet actions()}
		{#if canArchive && archived}
			<form
				method="POST"
				action="?/restore"
				use:enhance={() =>
					async ({ result, update }) => {
						await update();
						if (result.type === 'success') toast('Flight restored.');
					}}
			>
				<Button type="submit" kind="tertiary">Restore</Button>
			</form>
		{:else if canArchive && flight.session_id === null}
			<!-- Only a flight without a session can be archived. -->
			<Modal
				title="Archive"
				triggerKind="tertiary"
				submitLabel="Archive"
				action="?/archive"
				error={form?.archiveMessage}
				onsuccess={() => toast('Flight archived.')}
			>
				<p>
					The flight is left out of Flights and totals, as if it hadn’t happened. You can restore it
					later.
				</p>
			</Modal>
		{/if}
		<FlightStepper id={flight.id} />
	{/snippet}
</PageHeader>

<Grid rowGap>
	<Column narrow lg={11}>
		<TrackMap
			tracks={[flight.track]}
			areas={data.areas}
			token={data.mapboxToken}
			label="Map of the flight’s track"
		/>
	</Column>
	<Column lg={5}>
		<FlightFacts
			{flight}
			account={data.account}
			choices={data.choices}
			timeZone={data.timeZone}
			pilotMessage={form?.pilotMessage}
			groupMessage={form?.groupMessage}
		/>
	</Column>
</Grid>
