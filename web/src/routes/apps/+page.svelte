<script lang="ts">
	import { resolve } from '$app/paths';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import { dateTime } from '#lib/format.js';
	import AppDialog from './AppDialog.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	/** Where an app sends people back: who it is, as its name is only a label. */
	const host = (uri: string) => {
		try {
			return new URL(uri).host;
		} catch {
			return '';
		}
	};

	const rows = $derived(data.apps.map((app) => ({ ...app, host: host(app.redirect_uris[0]) })));
	type Row = (typeof rows)[number];

	const count = (value: number) => value.toLocaleString('en-US');
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{ key: 'host', header: 'Address', identifier: true },
		{ key: 'users', header: 'People', numeric: true, format: (app) => count(app.users) },
		{
			key: 'created_at',
			header: 'Added',
			wideOnly: true,
			format: (app) => dateTime(app.created_at, data.timeZone)
		}
	]);
</script>

<PageHeader title="Apps" />

{#snippet newApp()}
	<AppDialog title="New app" submitLabel="Add app" action="?/create" error={form?.message} />
{/snippet}

<Grid>
	<Column narrow>
		{#if data.apps.length === 0}
			<EmptyState
				title="No apps yet"
				body="Add the web apps your organisation runs alongside FleetRanger. People open them with their FleetRanger login, without being asked for permission, and see only what they can see here."
				action={newApp}
			/>
		{:else}
			<DataTable
				label="Apps"
				{columns}
				{rows}
				rowKey={(app) => app.id}
				rowHref={(app) => resolve('/apps/[id]', { id: app.id })}
				summary={`${count(data.apps.length)} ${data.apps.length === 1 ? 'app' : 'apps'}`}
			>
				{#snippet toolbar()}
					{@render newApp()}
				{/snippet}
			</DataTable>
		{/if}
	</Column>
</Grid>
