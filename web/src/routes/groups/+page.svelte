<script lang="ts">
	import { resolve } from '$app/paths';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
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
		<DataTable
			label="Groups"
			{columns}
			rows={data.groups}
			rowKey={(group) => group.id}
			rowHref={(group) => resolve('/groups/[id]', { id: group.id })}
			summary={`${count(data.groups.length)} ${data.groups.length === 1 ? 'group' : 'groups'} in the organisation`}
		>
			{#snippet toolbar()}
				{@render newGroup()}
			{/snippet}
		</DataTable>
		{#if data.groups.length === 1}
			<p class="note">
				Everything is in {data.groups[0].name}, and while it's the only group, the rest of the app
				doesn't show it. Groups hold flights, sessions and equipment, and give their members roles
				there; with a second group, each item and role belongs to one of them.
			</p>
		{/if}
	</Column>
</Grid>

<style>
	.note {
		margin: var(--cds-spacing-05) 0 0;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
	}
</style>
