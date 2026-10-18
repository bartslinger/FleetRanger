<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.pilots)[number];

	// The status column only while archived pilots are shown.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{ key: 'team_name', header: 'Team' },
		{ key: 'account_email', header: 'Linked user' },
		...(data.archived
			? [
					{
						key: 'archived_at',
						header: 'Status',
						format: (pilot: Row) => (pilot.archived_at ? 'Archived' : 'Active')
					} satisfies TableColumn<Row>
				]
			: [])
	]);

	const teamOptions = $derived(data.teams.map((team) => ({ value: team.id, label: team.name })));

	function showArchived(event: Event) {
		const checked = (event.currentTarget as HTMLInputElement).checked;
		goto(resolve(checked ? '/pilots?archived' : '/pilots'), {
			replaceState: true,
			reset: false
		});
	}
</script>

<PageHeader title="Pilots" />

{#snippet newPilot()}
	<Modal
		title="New pilot"
		submitLabel="Create"
		action="?/create"
		error={form && 'message' in form ? form.message : undefined}
		onsuccess={(created) => toast(`Pilot ${created?.created} created.`)}
	>
		<TextInput label="Name" name="name" required autocomplete="off" />
		<Select label="Team" name="team_id" options={teamOptions} required />
	</Modal>
{/snippet}

<Grid>
	<Column narrow>
		{#if data.teams.length === 0}
			<!-- Every pilot is in a team, so teams come first. -->
			<EmptyState
				title="Start by creating a team"
				body="Every pilot is in a team. Create a team first, then add its pilots here."
			/>
		{:else if data.pilots.length === 0 && !data.archived}
			<EmptyState
				title="No pilots yet"
				body="Pilots fly the drones. Link a pilot to a user so they can see their own and their team's flights."
				action={newPilot}
			/>
		{:else}
			<DataTable
				label="Pilots"
				{columns}
				rows={data.pilots}
				rowKey={(pilot) => pilot.id}
				rowHref={(pilot) => resolve('/pilots/[id]', { id: pilot.id })}
			>
				{#snippet toolbar()}
					<div class="filter">
						<Checkbox label="Show archived" checked={data.archived} onchange={showArchived} />
					</div>
					{@render newPilot()}
				{/snippet}
			</DataTable>
		{/if}
	</Column>
</Grid>

<style>
	.filter {
		display: flex;
		align-items: center;
		min-height: var(--cds-size-large);
		padding-inline: var(--cds-spacing-05);
	}
</style>
