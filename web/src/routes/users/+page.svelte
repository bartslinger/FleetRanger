<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Edit from '@carbon/icons/es/edit/16.js';
	import AppRoleGroups from '#lib/AppRoleGroups.svelte';
	import { allRoles, ORGANISATION_ROLES, ROLE_HELP, ROLE_LABELS } from '#lib/roles.js';
	import CheckboxGroup from '#lib/ui/CheckboxGroup.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import OverflowMenu from '#lib/ui/OverflowMenu.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import Toggle from '#lib/ui/Toggle.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.users)[number];

	// The groups by name only; their roles are on the user's page. A long
	// list is cut short so a row stays one line (carbon-expert's suggestion).
	function groupNames(user: Row): string | null {
		const names = user.groups.map((group) => group.name);
		if (names.length <= 3) return names.join(', ') || null;
		return `${names.slice(0, 2).join(', ')} and ${names.length - 2} more`;
	}

	// As a group's members (the organisation is the top group, user's
	// decision): name, roles, then the rest; the status column only while
	// archived users are shown.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{
			key: 'roles',
			header: 'Roles',
			wrap: true,
			format: (user) => allRoles(user.roles, user.app_roles, data.appRoles) || null
		},
		{ key: 'groups', header: 'Groups', wrap: true, format: groupNames },
		{ key: 'email', header: 'Login', wideOnly: true },
		...(data.archived
			? [
					{
						key: 'archived_at',
						header: 'Status',
						format: (user: Row) => (user.archived_at ? 'Archived' : 'Active')
					} satisfies TableColumn<Row>
				]
			: [])
	]);

	const roleOptions = ORGANISATION_ROLES.map((role) => ({
		value: role,
		label: ROLE_LABELS[role],
		helper: ROLE_HELP[role]
	}));

	// The last active admin keeps Admin, as on the user's page.
	const admins = $derived(
		data.users.filter((user) => user.archived_at === null && user.roles.includes('admin'))
	);
	const lastAdmin = (user: Row) => admins.length === 1 && admins[0].id === user.id;
	const rolesOf = (user: Row) =>
		roleOptions.map((option) =>
			option.value === 'admin' && lastAdmin(user)
				? {
						...option,
						helper: 'You are the only admin. Make another user an admin first.',
						disabled: true
					}
				: option
		);
	const rolesMessage = $derived(form && 'rolesMessage' in form ? form.rolesMessage : undefined);

	function showArchived(show: boolean) {
		goto(resolve(show ? '/users?archived' : '/users'), {
			replaceState: true,
			reset: false
		});
	}
</script>

<PageHeader title="Users" />

<!-- There is always at least one user: the admin looking at this page. -->
<Grid>
	<Column narrow>
		<DataTable
			label="Users"
			{columns}
			rows={data.users}
			rowKey={(user) => user.id}
			rowHref={(user) => resolve('/users/[id]', { id: user.id })}
			summary={`${data.users.length} ${data.users.length === 1 ? 'user' : 'users'}${data.archived ? ', archived included' : ''}`}
		>
			{#snippet toolbar()}
				<!-- On phones the toggle doesn't fit beside the count and New user,
				     so it moves into the overflow menu. -->
				<div class="filter">
					<Toggle
						label="Show archived"
						checked={data.archived}
						onchange={(event) => showArchived(event.currentTarget.checked)}
					/>
				</div>
				<div class="more">
					<OverflowMenu
						label="Options"
						variant="row"
						vertical
						items={[
							{
								label: 'Show archived',
								checked: data.archived,
								onselect: () => showArchived(!data.archived)
							}
						]}
					/>
				</div>
				<Modal
					title="New user"
					submitLabel="Create"
					action="?/create"
					error={form && 'message' in form ? form.message : undefined}
					onsuccess={(created) => toast(`${created?.created} created.`)}
				>
					<TextInput label="Name" name="name" required autocomplete="off" />
					<TextInput
						label="Email (optional)"
						name="email"
						type="email"
						autocomplete="off"
						helper="To log in with the Google account for this address. Leave empty for a pilot who doesn't use the website."
					/>
					<CheckboxGroup
						legend="FleetRanger roles"
						name="role"
						options={roleOptions}
						value={[]}
						helper="On the whole organisation. None for someone who only has roles in groups; add them to groups on their page."
					/>
					<AppRoleGroups appRoles={data.appRoles} value={[]} />
				</Modal>
			{/snippet}
			{#snippet rowAction(user)}
				<!-- As a group's members' Edit roles; archiving stays on the user's page. -->
				<Modal
					title="Edit roles"
					triggerIcon={Edit}
					triggerSize="md"
					triggerLabel="Edit roles of {user.name}"
					label="{user.name} in the organisation"
					submitLabel="Save"
					action="?/roles"
					error={rolesMessage}
					onsuccess={() => toast(`Roles of ${user.name} saved.`)}
				>
					<input type="hidden" name="user_id" value={user.id} />
					<CheckboxGroup
						legend="FleetRanger roles"
						name="role"
						options={rolesOf(user)}
						value={user.roles}
						helper="None for someone who only has roles in groups."
					/>
					<!-- A disabled checkbox isn't posted; Admin stays. -->
					{#if lastAdmin(user)}<input type="hidden" name="role" value="admin" />{/if}
					<AppRoleGroups appRoles={data.appRoles} value={user.app_roles} />
				</Modal>
			{/snippet}
		</DataTable>
	</Column>
</Grid>

<style>
	.filter {
		display: flex;
		align-items: center;
		min-height: var(--cds-size-large);
		padding-inline: var(--cds-spacing-05);
	}

	.more {
		display: none;
	}

	@media (max-width: 41.99rem) {
		.filter {
			display: none;
		}

		.more {
			display: flex;
		}
	}
</style>
