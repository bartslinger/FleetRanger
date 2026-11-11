<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Edit from '@carbon/icons/es/edit/16.js';
	import AppRoleGroups from '#lib/AppRoleGroups.svelte';
	import {
		allRoles,
		ORGANISATION_ROLES,
		rolesAcrossGroups,
		ROLE_HELP,
		ROLE_LABELS
	} from '#lib/roles.js';
	import type { GroupRole, OrganisationRole } from '#lib/server/backend.js';
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

	// While the organisation has only one group, nothing shows it: a user's
	// roles there and Admin are one set of roles (user's decision, 2026-11-11).
	const single = $derived(data.onlyGroup !== null);
	const inOnlyGroup = (user: Row) => user.groups.find((group) => group.id === data.onlyGroup);
	/** The roles the dialog shows checked. */
	const shownRoles = (user: Row): OrganisationRole[] => [
		...((inOnlyGroup(user)?.roles ?? []) as GroupRole[]),
		...user.roles
	];

	// With several groups: Admin, then each role with where it's held.
	function roleSummary(user: Row): string | null {
		if (single) {
			return allRoles(shownRoles(user), inOnlyGroup(user)?.app_roles ?? [], data.appRoles) || null;
		}
		return rolesAcrossGroups(user.roles.includes('admin'), user.groups, data.appRoles) || null;
	}

	// Name, roles, then the rest; the status column only while archived
	// users are shown.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{ key: 'roles', header: 'Roles', wrap: true, format: roleSummary },
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

	// With several groups only Admin is given here; the rest in the groups.
	const roleOptions = $derived(
		(single ? ORGANISATION_ROLES : (['admin'] as const)).map((role) => ({
			value: role,
			label: ROLE_LABELS[role],
			helper: ROLE_HELP[role]
		}))
	);

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
						helper={single
							? undefined
							: 'Other roles are given in groups, on their page once created.'}
					/>
					{#if single}<AppRoleGroups appRoles={data.appRoles} value={[]} />{/if}
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
						value={shownRoles(user)}
						helper={single ? undefined : 'Other roles are given in groups, on their page.'}
					/>
					<!-- A disabled checkbox isn't posted; Admin stays. -->
					{#if lastAdmin(user)}<input type="hidden" name="role" value="admin" />{/if}
					{#if single}
						<AppRoleGroups appRoles={data.appRoles} value={inOnlyGroup(user)?.app_roles ?? []} />
					{/if}
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
