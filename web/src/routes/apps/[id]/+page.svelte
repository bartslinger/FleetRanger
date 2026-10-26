<script lang="ts">
	import { resolve } from '$app/paths';
	import Edit from '@carbon/icons/es/edit/16.js';
	import TrashCan from '@carbon/icons/es/trash-can/16.js';
	import CodeSnippet from '#lib/ui/CodeSnippet.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime } from '#lib/format.js';
	import type { AppRole } from '#lib/server/backend.js';
	import AppDialog from '../AppDialog.svelte';
	import RoleDialog from '../RoleDialog.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const app = $derived(data.app);
	const message = $derived(form && 'message' in form ? form.message : undefined);
	const deleteMessage = $derived(form && 'deleteMessage' in form ? form.deleteMessage : undefined);
	const roleMessage = $derived(form && 'roleMessage' in form ? form.roleMessage : undefined);
	const deleteRoleMessage = $derived(
		form && 'deleteRoleMessage' in form ? form.deleteRoleMessage : undefined
	);
	const counted = (count: number) => `${count} ${count === 1 ? 'person' : 'people'}`;
	const people = $derived(counted(app.users));

	// The key in Plex Mono, as the app's code has it (carbon-expert's advice).
	const columns: TableColumn<AppRole>[] = [
		{ key: 'name', header: 'Name' },
		{ key: 'key', header: 'Key', code: true },
		{ key: 'holders', header: 'People', numeric: true, wideOnly: true }
	];
	const summary = $derived(`${app.roles.length} ${app.roles.length === 1 ? 'role' : 'roles'}`);
</script>

<PageHeader title={app.name} breadcrumb={[{ href: resolve('/apps'), label: 'Apps' }]}>
	{#snippet actions()}
		{#key app}
			<AppDialog
				title="Edit app"
				triggerKind="tertiary"
				submitLabel="Save"
				action="?/save"
				error={message}
				{app}
			/>
		{/key}
		<Modal
			title="Delete app"
			label={app.name}
			triggerKind="danger-tertiary"
			submitLabel="Delete app"
			danger
			action="?/delete"
			error={deleteMessage}
			onsuccess={() => toast(`App ${app.name} deleted.`)}
		>
			<p>
				{app.users > 0
					? `${people} use ${app.name}. Their access ends at once: the app can’t read anything from FleetRanger for them anymore.`
					: `People can no longer log in to ${app.name} with FleetRanger.`}
				Its roles go too, from everyone who holds them. Adding {app.name} again gives it a new client
				ID, and its roles have to be added and given again.
			</p>
		</Modal>
	{/snippet}
</PageHeader>

<Grid>
	<Column lg={8}>
		<dl class="facts">
			<!-- First: the one fact to act on (carbon-expert's advice). -->
			<div>
				<dt>Client ID</dt>
				<dd class="snippet">
					<CodeSnippet code={app.id} label="client ID" />
					<span class="helper">Copy this into the app’s settings.</span>
				</dd>
			</div>
			<div>
				<dt>Return addresses</dt>
				<dd>
					<ul>
						{#each app.redirect_uris as uri (uri)}<li>{uri}</li>{/each}
					</ul>
				</dd>
			</div>
			<div>
				<dt>People</dt>
				<dd>{app.users > 0 ? people : 'Nobody yet'}</dd>
			</div>
			<div>
				<dt>Added</dt>
				<dd>{dateTime(app.created_at, data.timeZone)}</dd>
			</div>
		</dl>
	</Column>
	<Column narrow>
		<h2 class="section">Roles</h2>
		{#key app}
			{#if app.roles.length === 0}
				<!-- Replaces the table, and offers its action (Carbon's empty states). -->
				<EmptyState
					title="No roles yet"
					body="Roles say what people may do in {app.name}. Add the roles its instructions name, then give them to people on the Users and Groups pages."
				>
					{#snippet action()}<RoleDialog app={app.name} error={roleMessage} />{/snippet}
				</EmptyState>
			{:else}
				<DataTable label="Roles" {columns} rows={app.roles} rowKey={(role) => role.id} {summary}>
					{#snippet toolbar()}<RoleDialog app={app.name} error={roleMessage} />{/snippet}
					{#snippet rowAction(role)}
						<span class="actions">
							<RoleDialog app={app.name} error={roleMessage} {role} triggerIcon={Edit} />
							<!-- A danger modal without typing the name: a role is quick to
							     add and give again (carbon-expert's advice). -->
							<Modal
								title="Delete role"
								label={role.name}
								triggerIcon={TrashCan}
								triggerSize="md"
								triggerLabel="Delete role {role.name}"
								submitLabel="Delete role"
								danger
								action="?/deleteRole"
								error={deleteRoleMessage}
								onsuccess={() => toast(`Role ${role.name} deleted.`)}
							>
								<input type="hidden" name="id" value={role.id} />
								<p>
									{role.holders > 0
										? `${counted(role.holders)} lose this role, and ${app.name} no longer sees it for them.`
										: 'Nobody holds this role.'}
									Adding it again makes a new role, to give to people again.
								</p>
							</Modal>
						</span>
					{/snippet}
				</DataTable>
			{/if}
		{/key}
	</Column>
</Grid>

<style>
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-07);
	}

	dt,
	.helper {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.helper {
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		overflow-wrap: anywhere;
	}

	.snippet {
		display: grid;
		/* Lets the snippet narrow, scrolling the ID, instead of running off phones. */
		grid-template-columns: minmax(0, 1fr);
		gap: var(--cds-spacing-02);
		margin-top: var(--cds-spacing-03);
	}

	ul {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.section {
		margin-bottom: var(--cds-spacing-05);
		/* The column has no gutter; line up with the table's text. */
		padding-inline: var(--cds-spacing-05);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}

	.actions {
		display: inline-flex;
	}
</style>
