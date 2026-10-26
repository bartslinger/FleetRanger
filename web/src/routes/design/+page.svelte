<!-- Every component in its states, to review the design in both themes. -->
<script lang="ts">
	import Button from '#lib/ui/Button.svelte';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Pagination from '#lib/ui/Pagination.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import NumberInput from '#lib/ui/NumberInput.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import Tag from '#lib/ui/Tag.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import Toggle from '#lib/ui/Toggle.svelte';

	interface ExampleFlight {
		id: string;
		takeoff: string;
		pilot: string;
		drone: string;
		airTime: string;
		distance: string;
	}

	// Made-up example rows.
	const flights: ExampleFlight[] = [
		{
			id: '1',
			takeoff: '17 Oct 2026, 06:12',
			pilot: 'Pilot A',
			drone: 'Mavic 3T',
			airTime: '0:38:12',
			distance: '12.4 km'
		},
		{
			id: '2',
			takeoff: '17 Oct 2026, 06:58',
			pilot: 'Pilot A',
			drone: 'Mavic 3T',
			airTime: '0:41:05',
			distance: '14.9 km'
		},
		{
			id: '3',
			takeoff: '16 Oct 2026, 19:40',
			pilot: 'Pilot B',
			drone: 'Mavic 3E',
			airTime: '0:07:44',
			distance: '1.2 km'
		}
	];

	const columns: TableColumn<ExampleFlight>[] = [
		{ key: 'takeoff', header: 'Takeoff', sortable: true },
		{ key: 'pilot', header: 'Pilot' },
		{ key: 'drone', header: 'Drone' },
		{ key: 'airTime', header: 'Air time', numeric: true },
		{ key: 'distance', header: 'Distance', numeric: true }
	];

	const tagTypes = [
		'gray',
		'cool-gray',
		'warm-gray',
		'red',
		'magenta',
		'purple',
		'blue',
		'cyan',
		'teal',
		'green'
	] as const;

	let name = $state('');
	let group = $state('north');
	let batteries = $state(4);
</script>

<PageHeader title="Components" description="Every component in its states. Only on the dev server.">
	{#snippet actions()}
		<Button kind="tertiary" size="md">Secondary action</Button>
		<Button size="md">Primary action</Button>
	{/snippet}
</PageHeader>

<div class="sections">
	<Grid>
		<Column md={6} lg={8}>
			<section>
				<h2>Grid</h2>
				<p>
					Text, buttons and labelled fields sit on the columns (wide gutter mode). Tables, tiles and
					notifications hang 16px into the gutter (narrow mode), so the text inside them lines up
					with this text.
				</p>
			</section>
		</Column>
	</Grid>

	<Grid>
		<Column>
			<section>
				<h2>Button</h2>
				<div class="row">
					<Button>Primary</Button>
					<Button kind="secondary">Secondary</Button>
					<Button kind="tertiary">Tertiary</Button>
					<Button kind="ghost">Ghost</Button>
					<Button kind="danger">Danger</Button>
				</div>
				<div class="row">
					<Button size="md">Medium</Button>
					<Button size="sm">Small</Button>
					<Button href="#top" kind="secondary" size="sm">Link</Button>
					<Button disabled>Disabled</Button>
					<Button kind="tertiary" disabled>Disabled</Button>
					<Button kind="ghost" disabled>Disabled</Button>
				</div>
			</section>
		</Column>
	</Grid>

	<Grid>
		<Column>
			<section>
				<h2>Tag</h2>
				<div class="row">
					{#each tagTypes as type (type)}
						<Tag {type}>{type}</Tag>
					{/each}
				</div>
				<div class="row">
					<Tag size="sm" type="green">Synced</Tag>
					<Tag size="sm" type="red">Failed</Tag>
				</div>
			</section>
		</Column>
	</Grid>

	<Grid>
		<Column>
			<h2>Inline notification</h2>
		</Column>
		<Column narrow lg={12}>
			<div class="stack">
				<InlineNotification kind="error" title="Log could not be read.">
					The file ends before the last record.
				</InlineNotification>
				<InlineNotification kind="warning" title="DJI is not responding.">
					Logs will be processed when it is back.
				</InlineNotification>
				<InlineNotification kind="success" title="Flight moved to North." />
				<InlineNotification title="12 flights have no pilot." />
			</div>
		</Column>
	</Grid>

	<Grid rowGap>
		<Column>
			<h2>Text input and select</h2>
		</Column>
		<Column md={4}>
			<TextInput label="Pilot name" placeholder="First and last name" bind:value={name} />
		</Column>
		<Column md={4}>
			<TextInput label="Callsign" helper="Shown on the remote controller." />
		</Column>
		<Column md={4}>
			<TextInput label="Email" value="pilot@" error="Enter a complete email address." />
		</Column>
		<Column md={4}>
			<TextInput label="Disabled" value="Read only" disabled />
		</Column>
		<Column md={4}>
			<Select
				label="Group"
				bind:value={group}
				options={[
					{ value: 'north', label: 'North' },
					{ value: 'south', label: 'South' }
				]}
			/>
		</Column>
		<Column md={4}>
			<Select
				label="Drone"
				error="Choose a drone."
				options={[{ value: '', label: 'Choose a drone' }]}
			/>
		</Column>
		<Column md={4}>
			<div class="row">
				<Checkbox label="Show archived" />
				<Checkbox label="Checked" checked />
				<Checkbox label="Disabled" disabled />
			</div>
			<div class="row">
				<Toggle label="Show archived" />
				<Toggle label="On" checked />
				<Toggle label="Disabled" disabled />
			</div>
		</Column>
	</Grid>

	<Grid rowGap>
		<Column>
			<h2>Number input</h2>
		</Column>
		<Column md={4}>
			<NumberInput
				label="Batteries"
				helper="Enter a number from 0 to 12."
				min={0}
				max={12}
				bind:value={batteries}
			/>
		</Column>
		<Column md={4}>
			<NumberInput label="Small" size="sm" min={0} value={0} />
		</Column>
		<Column md={4}>
			<NumberInput label="Flights" value={0} error="Enter a whole number of 1 or more." />
		</Column>
		<Column md={4}>
			<NumberInput label="Read only" value={7} readonly />
		</Column>
		<Column md={4}>
			<NumberInput label="Disabled" value={2} disabled />
		</Column>
	</Grid>

	<Grid>
		<Column>
			<h2>Data table</h2>
		</Column>
		<Column narrow>
			<div class="stack">
				<div>
					<DataTable
						label="Example flights"
						{columns}
						rows={flights}
						rowKey={(flight) => flight.id}
						sort={{ key: 'takeoff', descending: true }}
						sortHref={() => '#'}
					/>
					<Pagination
						page={1}
						size={25}
						total={3}
						sizes={[25, 50, 100]}
						noun="flights"
						href={() => '#'}
					/>
				</div>
				<DataTable
					label="Example flights, small, the second current"
					{columns}
					rows={flights}
					rowKey={(flight) => flight.id}
					size="sm"
					currentRow="2"
				>
					{#snippet cell(flight, column)}
						{#if column.key === 'pilot'}
							<Tag size="sm" type="blue">{flight.pilot}</Tag>
						{:else}
							{flight[column.key]}
						{/if}
					{/snippet}
				</DataTable>
				<EmptyState
					title="No flights yet"
					body="Flights appear here when flight logs from your remote controllers are imported."
				/>
			</div>
		</Column>
	</Grid>
</div>

<style>
	.sections {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-09);
	}

	section,
	.stack {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
	}

	/* This page's own section titles, not a pattern for other pages. */
	h2 {
		margin-bottom: var(--cds-spacing-05);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}

	section h2 {
		margin-bottom: 0;
	}

	p {
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--cds-spacing-05);
	}
</style>
