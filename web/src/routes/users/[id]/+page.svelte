<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { back } from '#lib/back.svelte.js';
	import { dateTime } from '#lib/format.js';
	import { ORGANISATION_ROLES, ROLE_HELP, ROLE_LABELS, roleList } from '#lib/roles.js';
	import Button from '#lib/ui/Button.svelte';
	import CheckboxGroup from '#lib/ui/CheckboxGroup.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);
	// Save and Cancel go back to where the user came from.
	const previous = back(() => resolve('/users'));

	const user = $derived(data.user);
	const archived = $derived(user.archived_at !== null);
	// Admins can't remove their own login, or archive or delete themselves;
	// the backend refuses it too.
	const self = $derived(data.account?.id === user.id);
	const message = $derived(form && 'message' in form ? form.message : undefined);

	// The last active admin keeps Admin and isn't archived: disabled, not
	// hidden, since it changes once someone else is an admin (carbon-expert's
	// advice, from Carbon's disabled states).
	const roleOptions = $derived(
		ORGANISATION_ROLES.map((role) => ({
			value: role,
			label: ROLE_LABELS[role],
			helper: ROLE_HELP[role],
			disabled: role === 'admin' && data.lastAdmin
		}))
	);

	type Membership = (typeof data.user.groups)[number];
	const groupColumns: TableColumn<Membership>[] = [
		{ key: 'name', header: 'Group' },
		{ key: 'roles', header: 'Roles', wrap: true, format: (group) => roleList(group.roles) }
	];
</script>

<PageHeader
	title={user.name}
	breadcrumb={[{ href: resolve('/users'), label: 'Users' }]}
	status={archived ? 'Archived' : undefined}
>
	{#snippet actions()}
		<!-- Apart from the form below: these apply at once. -->
		{#if archived}
			<form
				method="POST"
				action="?/restore"
				use:enhance={() =>
					async ({ result, update }) => {
						await update();
						if (result.type === 'success') toast(`${user.name} restored.`);
					}}
			>
				<Button type="submit" kind="tertiary">Restore user</Button>
			</form>
		{:else if data.lastAdmin && !self}
			<Button kind="tertiary" disabled>Archive user</Button>
		{:else if !self}
			<Modal
				title="Archive user"
				triggerKind="tertiary"
				submitLabel="Archive user"
				action="?/archive"
				error={form && 'archiveMessage' in form ? form.archiveMessage : undefined}
				onsuccess={() => toast(`${user.name} archived.`)}
			>
				<p>
					{user.name} is hidden from lists, is logged out everywhere, can't log in and can't be chosen
					as a pilot. Their flights stay as they are. You can restore them later.
				</p>
			</Modal>
		{/if}
		{#if !self && user.flights === 0}
			<Modal
				title="Delete user"
				triggerKind="danger-tertiary"
				submitLabel="Delete user"
				danger
				action="?/delete"
				error={form && 'deleteMessage' in form ? form.deleteMessage : undefined}
				onsuccess={() => toast(`${user.name} deleted.`)}
			>
				<p>
					{user.name} has no flights, so they can be deleted for good, for example if they were added
					by mistake. This can't be undone.
				</p>
			</Modal>
		{/if}
	{/snippet}
</PageHeader>

<Grid rowGap>
	{#if data.lastAdmin}
		<Column narrow lg={8}>
			<InlineNotification
				kind="warning"
				title={self ? 'You are the only admin.' : `${user.name} is the only admin.`}
			>
				{self
					? 'Make another user an admin first. Then you can give up this role.'
					: 'Make another user an admin first. Then you can remove this role or archive this user.'}
			</InlineNotification>
		</Column>
	{/if}
	{#if message}
		<Column narrow lg={8}>
			<InlineNotification kind="error" title={message} />
		</Column>
	{/if}
	<Column lg={8}>
		<dl class="facts">
			<div>
				<dt>Flights as pilot</dt>
				<dd>{user.flights}</dd>
			</div>
			<div>
				<dt>Last login</dt>
				<dd>
					{user.last_login_at ? dateTime(user.last_login_at, data.timeZone) : 'Never'}
				</dd>
			</div>
		</dl>
		<form
			class="form"
			method="POST"
			action="?/save"
			use:enhance={() => {
				busy = true;
				return async ({ result, update }) => {
					busy = false;
					if (result.type !== 'success') return update({ reset: false });
					await previous.go();
					toast(`${user.name} saved.`);
				};
			}}
		>
			<TextInput label="Name" name="name" value={user.name} required autocomplete="off" />
			<TextInput
				label={self ? 'Email' : 'Email (optional)'}
				name="email"
				type="email"
				value={user.email ?? ''}
				required={self}
				autocomplete="off"
				helper={self
					? 'You log in with this address.'
					: 'To log in with the Google account for this address. Empty: no login.'}
			/>
			<CheckboxGroup
				legend="Roles on the whole organisation"
				name="role"
				options={roleOptions}
				value={user.roles}
				helper="None for someone who only has roles in groups."
			/>
			<!-- A disabled checkbox isn't posted; Admin stays. -->
			{#if data.lastAdmin}<input type="hidden" name="role" value="admin" />{/if}
			<div class="buttons">
				<Button type="submit" disabled={busy} fill>Save</Button>
				<Button kind="secondary" href={previous.href} onclick={previous.onclick} fill>Cancel</Button
				>
			</div>
		</form>
	</Column>
	<Column narrow>
		<h2 class="section">Groups</h2>
		{#if user.groups.length === 0}
			<p class="note">Not in any group. Members are added on a group’s page.</p>
		{:else}
			<p class="note">Roles in a group are changed on the group’s page.</p>
			<DataTable
				label="Groups"
				columns={groupColumns}
				rows={user.groups}
				rowKey={(group) => group.id}
				rowHref={(group) => resolve('/groups/[id]', { id: group.id })}
			/>
		{/if}
	</Column>
</Grid>

<style>
	/* 32px before the form, as between fields. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-07);
	}

	dt {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* Carbon's in-page form: fields 32px apart, 48px before the buttons. */
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	/* Carbon's fluid pair: equal halves of the fields' width, 1px apart. */
	.buttons {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		margin-top: var(--cds-spacing-05);
	}

	.section {
		margin-top: var(--cds-spacing-07);
		margin-bottom: var(--cds-spacing-03);
		/* The column has no gutter; line up with the table's text. */
		padding-inline: var(--cds-spacing-05);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}

	.note {
		margin: 0 0 var(--cds-spacing-05);
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
	}
</style>
