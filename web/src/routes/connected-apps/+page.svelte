<script lang="ts">
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime } from '#lib/format.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.connections)[number];

	/** What each scope lets an app do, briefly. */
	const ACCESS: Record<string, string> = {
		'flights:read': 'Reads flights, sessions and comments',
		'sessions:list': 'Reads sessions',
		'sessions:read': 'Reads comments',
		'flights:list': 'Reads flights',
		'groups:list': 'Reads groups',
		'areas:list': 'Reads areas',
		'equipment:list': 'Reads equipment'
	};

	// On phones only the app and its address, which tells apps with the same
	// name apart, so Disconnect stays in view.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'app', header: 'App' },
		{ key: 'host', header: 'Address', identifier: true },
		{
			key: 'scopes',
			header: 'Access',
			wideOnly: true,
			format: (row) => row.scopes.map((scope) => ACCESS[scope] ?? scope).join(', ')
		},
		{
			key: 'created_at',
			header: 'Connected',
			wideOnly: true,
			format: (row) => dateTime(row.created_at, data.timeZone)
		},
		{
			key: 'last_used_at',
			header: 'Last used',
			wideOnly: true,
			format: (row) => dateTime(row.last_used_at, data.timeZone)
		}
	]);
</script>

<PageHeader
	title="Connected apps"
	description="Apps you allowed to act for you, such as AI assistants. Disconnecting one stops its access at once."
/>

<Grid>
	<Column narrow>
		{#if data.connections.length === 0}
			<EmptyState
				title="No connected apps"
				body="To connect an AI assistant, add FleetRanger in the assistant's settings, for example as a connector in Claude. It then asks you here to allow it."
			/>
		{:else}
			<DataTable label="Connected apps" {columns} rows={data.connections} rowKey={(row) => row.id}>
				{#snippet rowAction(row)}
					<Modal
						title="Disconnect"
						triggerKind="danger-ghost"
						triggerLabel="Disconnect {row.app} ({row.host})"
						label="{row.app} · {row.host}"
						submitLabel="Disconnect"
						danger
						action="?/disconnect"
						error={form && 'message' in form ? form.message : undefined}
						onsuccess={() => toast(`${row.app} disconnected.`)}
					>
						<input type="hidden" name="id" value={row.id} />
						<input type="hidden" name="app" value={row.app} />
						<p>
							{row.app} can no longer act for you. To use it again, connect it again from the app.
						</p>
					</Modal>
				{/snippet}
			</DataTable>
		{/if}
	</Column>
</Grid>
