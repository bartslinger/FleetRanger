<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { back } from '#lib/back.svelte.js';
	import { capacity, dateTime, duration, health } from '#lib/format.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import TimeChart from '#lib/ui/TimeChart.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	type Row = (typeof data.battery.flight_list)[number];

	const battery = $derived(data.battery);
	const list = resolve('/equipment?tab=batteries');
	const message = $derived(form && 'message' in form ? form.message : undefined);
	// Save and Cancel go back to where the user came from.
	const previous = back(() => list);

	// The drones it flew in, most recent first.
	const drones = $derived([
		...new Set(
			battery.flight_list.map((flight) => flight.aircraft_name ?? flight.aircraft_serial ?? '')
		)
	]);

	// A point per flight, also of flights the user doesn't see: the battery's
	// wear. Oldest first, as the chart reads.
	const readings = $derived(
		battery.capacity_readings.map((reading) => ({
			at: reading.takeoff_at,
			value: reading.full_capacity_mah
		}))
	);

	const percent = (value: number | null) => (value === null ? '—' : `${value}%`);

	// The battery's columns first, so they show on phones before the table
	// scrolls; units in the headers keep the columns narrow (carbon-expert's
	// advice).
	const columns: TableColumn<Row>[] = $derived([
		{
			key: 'takeoff_at',
			header: 'Takeoff',
			format: (flight) => dateTime(flight.takeoff_at, data.timeZone)
		},
		{
			key: 'charge_start_pct',
			header: 'Charge',
			numeric: true,
			format: (flight) =>
				flight.charge_start_pct === null && flight.charge_end_pct === null
					? null
					: `${percent(flight.charge_start_pct)} → ${percent(flight.charge_end_pct)}`
		},
		{
			key: 'full_capacity_mah',
			header: 'Capacity (mAh)',
			numeric: true,
			format: (flight) => flight.full_capacity_mah?.toLocaleString('en-US')
		},
		{
			key: 'temperature_max_c',
			header: 'Max temperature (°C)',
			numeric: true,
			format: (flight) =>
				flight.temperature_max_c === null ? null : String(Math.round(flight.temperature_max_c))
		},
		{
			key: 'cycles',
			header: 'Cycles',
			numeric: true,
			format: (flight) => flight.cycles?.toLocaleString('en-US')
		},
		{
			key: 'air_time_s',
			header: 'Air time',
			numeric: true,
			format: (flight) => duration(flight.air_time_s)
		},
		{
			key: 'aircraft_serial',
			header: 'Drone',
			identifier: true,
			format: (flight) => flight.aircraft_name ?? flight.aircraft_serial
		}
	]);
</script>

<PageHeader
	title={battery.name ?? battery.serial}
	breadcrumb={[{ href: list, label: 'Equipment' }]}
/>

<Grid rowGap>
	<Column lg={8}>
		<!-- In the form's column, so it stacks above it at every width. -->
		{#if message}
			<div class="message"><InlineNotification kind="error" title={message} /></div>
		{/if}
		<!-- Read-only facts, from the flight logs: the battery as of its latest flight. -->
		<dl class="facts">
			<div>
				<dt>Kind</dt>
				<dd>Battery</dd>
			</div>
			<div>
				<dt>Serial number</dt>
				<dd class="identifier">{battery.serial}</dd>
			</div>
			<div>
				<dt>Cycles</dt>
				<dd>{battery.cycles?.toLocaleString('en-US') ?? 'Unknown'}</dd>
			</div>
			<!-- What it holds now, as a share of what it held new. -->
			<div>
				<dt>Health</dt>
				<dd>{health(battery.full_capacity_mah, battery.design_capacity_mah) ?? 'Unknown'}</dd>
			</div>
			<div>
				<dt>Full charge capacity</dt>
				<dd>{battery.full_capacity_mah ? capacity(battery.full_capacity_mah) : 'Unknown'}</dd>
			</div>
			<div>
				<dt>Design capacity</dt>
				<dd>{battery.design_capacity_mah ? capacity(battery.design_capacity_mah) : 'Unknown'}</dd>
			</div>
			<div>
				<dt>{drones.length === 1 ? 'Drone' : 'Drones'}</dt>
				<dd>{drones.join(', ')}</dd>
			</div>
			<div>
				<dt>Last flight</dt>
				<dd>{dateTime(battery.last_flight_at, data.timeZone)}</dd>
			</div>
		</dl>
		{#if data.edit}
			<form
				class="form"
				method="POST"
				action="?/save"
				use:enhance={() => {
					busy = true;
					return async ({ result, update }) => {
						busy = false;
						if (result.type !== 'success') return update({ reset: false });
						await previous.go();
						toast('Battery saved.');
					};
				}}
			>
				<TextInput
					label="Name (optional)"
					name="name"
					value={battery.name ?? ''}
					autocomplete="off"
					helper="For example the number written on it."
				/>
				<div class="buttons">
					<Button type="submit" disabled={busy} fill>Save</Button>
					<Button kind="secondary" href={previous.href} onclick={previous.onclick} fill
						>Cancel</Button
					>
				</div>
			</form>
		{/if}
	</Column>
	{#if readings.length > 0}
		<Column narrow>
			<TimeChart
				title="Full charge capacity over time"
				xTitle="Flight date"
				yTitle="Full charge capacity (mAh)"
				{readings}
				formatValue={capacity}
				formatTime={(at) => dateTime(at, data.timeZone)}
				description="Line chart of the battery’s full charge capacity in mAh by flight date, over all its flights. The flights table below lists the values of the flights you can see."
			/>
		</Column>
	{/if}
	<Column narrow>
		<DataTable
			label="Flights with this battery"
			{columns}
			rows={battery.flight_list}
			rowKey={(flight) => flight.flight_id}
			rowHref={(flight) => resolve('/flights/[id]', { id: flight.flight_id })}
			summary="{battery.flight_list.length} {battery.flight_list.length === 1
				? 'flight'
				: 'flights'}"
		/>
	</Column>
</Grid>

<style>
	.message {
		margin-bottom: var(--cds-spacing-07);
	}

	/* As on a drone's page: facts 16px apart, 32px before the form. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-07);
	}

	dt {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.identifier {
		font-variant-numeric: tabular-nums slashed-zero;
	}

	/* Carbon's in-page form: fields 32px apart, 48px before the buttons. */
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	/* Carbon's fluid pair: equal halves of the fields' width, 1px apart. */
	.buttons {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		margin-top: var(--cds-spacing-05);
	}
</style>
