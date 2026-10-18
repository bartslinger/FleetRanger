<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.teams)[number];

	// The status column only while archived teams are shown.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{ key: 'pilots', header: 'Pilots', numeric: true },
		...(data.archived
			? [
					{
						key: 'archived_at',
						header: 'Status',
						format: (team: Row) => (team.archived_at ? 'Archived' : 'Active')
					} satisfies TableColumn<Row>
				]
			: [])
	]);

	function showArchived(event: Event) {
		const checked = (event.currentTarget as HTMLInputElement).checked;
		goto(resolve(checked ? '/teams?archived' : '/teams'), {
			replaceState: true,
			reset: false
		});
	}
</script>

<PageHeader title="Teams" />

{#snippet newTeam()}
	<Modal
		title="New team"
		submitLabel="Create"
		action="?/create"
		error={form && 'message' in form ? form.message : undefined}
	>
		<TextInput label="Name" name="name" required autocomplete="off" />
	</Modal>
{/snippet}

<Grid rowGap>
	{#if form && 'created' in form}
		<Column narrow>
			<InlineNotification kind="success" title="Team {form.created} created." />
		</Column>
	{/if}
	<Column narrow>
		{#if data.teams.length === 0 && !data.archived}
			<EmptyState
				title="No teams yet"
				body="Pilots work in teams. Each flight belongs to its pilot's team, and team members see each other's flights."
				action={newTeam}
			/>
		{:else}
			<DataTable
				label="Teams"
				{columns}
				rows={data.teams}
				rowKey={(team) => team.id}
				rowHref={(team) => resolve('/teams/[id]', { id: team.id })}
			>
				{#snippet toolbar()}
					<div class="filter">
						<Checkbox label="Show archived" checked={data.archived} onchange={showArchived} />
					</div>
					{@render newTeam()}
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
