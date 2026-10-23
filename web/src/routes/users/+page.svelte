<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ORGANISATION_ROLES, ROLE_HELP, ROLE_LABELS, roleList } from '#lib/roles.js';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import CheckboxGroup from '#lib/ui/CheckboxGroup.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
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

	// The status column only while archived users are shown.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{ key: 'email', header: 'Login' },
		{ key: 'roles', header: 'Roles', wrap: true, format: (user) => roleList(user.roles) || null },
		{ key: 'groups', header: 'Groups', wrap: true, format: groupNames },
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

	function showArchived(event: Event) {
		const checked = (event.currentTarget as HTMLInputElement).checked;
		goto(resolve(checked ? '/users?archived' : '/users'), {
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
		>
			{#snippet toolbar()}
				<div class="filter">
					<Checkbox label="Show archived" checked={data.archived} onchange={showArchived} />
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
						legend="Roles on the whole organisation"
						name="role"
						options={roleOptions}
						value={[]}
						helper="None for someone who only has roles in groups. Add them to groups on a group’s page."
					/>
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
</style>
