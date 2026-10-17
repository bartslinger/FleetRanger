<!-- Every component in its states, to review the design in both themes. -->
<script lang="ts">
	import Button from '#lib/ui/Button.svelte';
	import DataTable, { type Column } from '#lib/ui/DataTable.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import Tag from '#lib/ui/Tag.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';

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

	const columns: Column<ExampleFlight>[] = [
		{ key: 'takeoff', header: 'Takeoff' },
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
	let team = $state('north');
</script>

<PageHeader title="Components" description="Every component in its states. Only on the dev server.">
	{#snippet actions()}
		<Button kind="tertiary" size="md">Secondary action</Button>
		<Button size="md">Primary action</Button>
	{/snippet}
</PageHeader>

<div class="sections">
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

	<section>
		<h2>Inline notification</h2>
		<div class="stack">
			<InlineNotification kind="error" title="Log could not be read.">
				The file ends before the last record.
			</InlineNotification>
			<InlineNotification kind="warning" title="DJI is not responding.">
				Logs will be processed when it is back.
			</InlineNotification>
			<InlineNotification kind="success" title="Flight moved to Team North." />
			<InlineNotification title="12 flights have no pilot." />
		</div>
	</section>

	<section>
		<h2>Text input and select</h2>
		<div class="form">
			<TextInput label="Pilot name" placeholder="First and last name" bind:value={name} />
			<TextInput label="Callsign" helper="Shown on the remote controller." />
			<TextInput label="Email" value="pilot@" error="Enter a complete email address." />
			<TextInput label="Disabled" value="Read only" disabled />
			<Select
				label="Team"
				bind:value={team}
				options={[
					{ value: 'north', label: 'Team North' },
					{ value: 'south', label: 'Team South' }
				]}
			/>
			<Select
				label="Drone"
				error="Choose a drone."
				options={[{ value: '', label: 'Choose a drone' }]}
			/>
		</div>
	</section>

	<section>
		<h2>Data table</h2>
		<div class="stack">
			<DataTable {columns} rows={flights} rowKey={(flight) => flight.id} />
			<DataTable {columns} rows={flights} rowKey={(flight) => flight.id} size="sm">
				{#snippet cell(flight, column)}
					{#if column.key === 'pilot'}
						<Tag size="sm" type="blue">{flight.pilot}</Tag>
					{:else}
						{flight[column.key]}
					{/if}
				{/snippet}
			</DataTable>
			<DataTable {columns} rows={[]} rowKey={(flight) => flight.id} empty="No flights yet." />
		</div>
	</section>
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
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--cds-spacing-05);
	}

	.form {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
		gap: var(--cds-spacing-06) var(--cds-spacing-05);
		align-items: start;
	}
</style>
