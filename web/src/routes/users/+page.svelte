<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ROLE_HELP, ROLE_LABELS, ROLE_OPTIONS, type Role } from '#lib/roles.js';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import RadioButtonGroup from '#lib/ui/RadioButtonGroup.svelte';
	import CheckboxGroup from '#lib/ui/CheckboxGroup.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.users)[number];

	let role: Role = $state('member');

	// The status column only while archived users are shown.
	const columns: TableColumn<Row>[] = $derived([
		{ key: 'name', header: 'Name' },
		{
			key: 'team_names',
			header: 'Teams',
			wrap: true,
			format: (user) => user.team_names.join(', ')
		},
		{ key: 'email', header: 'Login' },
		{ key: 'role', header: 'Role', format: (user) => ROLE_LABELS[user.role] },
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

	const teamOptions = $derived(data.teams.map((team) => ({ value: team.id, label: team.name })));

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
					<CheckboxGroup
						legend="Teams"
						name="team_id"
						options={teamOptions}
						value={[]}
						helper="The teams they fly for; none for someone who doesn't fly."
					/>
					<TextInput
						label="Email (optional)"
						name="email"
						type="email"
						autocomplete="off"
						helper="To log in with the Google account for this address. Leave empty for a pilot who doesn't use the website."
					/>
					<RadioButtonGroup legend="Role" name="role" options={ROLE_OPTIONS} bind:value={role} />
					<p class="help">{ROLE_HELP}</p>
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

	.help {
		margin-top: calc(-1 * var(--cds-spacing-03));
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
