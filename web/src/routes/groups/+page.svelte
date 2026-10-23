<script lang="ts">
	import { resolve } from '$app/paths';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.groups)[number];

	const count = (value: number) => value.toLocaleString('en-US');
	const columns: TableColumn<Row>[] = [
		{ key: 'name', header: 'Name' },
		{ key: 'members', header: 'Members', numeric: true, format: (group) => count(group.members) },
		{ key: 'flights', header: 'Flights', numeric: true, format: (group) => count(group.flights) }
	];
</script>

<PageHeader title="Groups" />

{#snippet newGroup()}
	<Modal
		title="New group"
		submitLabel="Create"
		action="?/create"
		error={form && 'message' in form ? form.message : undefined}
		onsuccess={(created) => toast(`Group ${created?.created} created.`)}
	>
		<TextInput label="Name" name="name" required autocomplete="off" />
	</Modal>
{/snippet}

<Grid>
	<Column narrow>
		{#if data.groups.length === 0}
			<EmptyState
				title="No groups yet"
				body="Groups hold flights, sessions and equipment, and give their members roles there. A small organisation doesn't need them: without groups, everything is in the organisation."
				action={newGroup}
			/>
		{:else}
			<DataTable
				label="Groups"
				{columns}
				rows={data.groups}
				rowKey={(group) => group.id}
				rowHref={(group) => resolve('/groups/[id]', { id: group.id })}
				summary={`${count(data.groups.length)} ${data.groups.length === 1 ? 'group' : 'groups'}`}
			>
				{#snippet toolbar()}
					{@render newGroup()}
				{/snippet}
			</DataTable>
		{/if}
	</Column>
</Grid>
