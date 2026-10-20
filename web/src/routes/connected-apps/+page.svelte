<script lang="ts">
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime } from '#lib/format.js';
	import { MediaQuery } from 'svelte/reactivity';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.connections)[number];

	/** What each scope lets an app do, briefly. */
	const ACCESS: Record<string, string> = { 'flights:read': 'Reads flights and sessions' };

	// On phones only the app and its address, so Disconnect stays in view. The address tells apps with the same name apart.
	const wide = new MediaQuery('min-width: 42rem');
	const address: TableColumn<Row> = { key: 'host', header: 'Address', identifier: true };
	const lastUsed: TableColumn<Row> = $derived({
		key: 'last_used_at',
		header: `Last used (${data.timeZoneName})`,
		format: (row) => dateTime(row.last_used_at, data.timeZone)
	});
	const columns: TableColumn<Row>[] = $derived(
		wide.current
			? [
					{ key: 'app', header: 'App' },
					address,
					{
						key: 'scopes',
						header: 'Access',
						format: (row) => row.scopes.map((scope) => ACCESS[scope] ?? scope).join(', ')
					},
					{
						key: 'created_at',
						header: `Connected (${data.timeZoneName})`,
						format: (row) => dateTime(row.created_at, data.timeZone)
					},
					lastUsed
				]
			: [{ key: 'app', header: 'App' }, address]
	);
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
				body="Apps appear here once you allow them to connect to FleetRanger."
			/>
		{:else}
			<DataTable label="Connected apps" {columns} rows={data.connections} rowKey={(row) => row.id}>
				{#snippet rowAction(row)}
					<Modal
						title="Disconnect"
						triggerKind="danger-tertiary"
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
