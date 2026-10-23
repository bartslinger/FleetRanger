<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import Edit from '@carbon/icons/es/edit/16.js';
	import SubtractAlt from '@carbon/icons/es/subtract--alt/16.js';
	import { back } from '#lib/back.svelte.js';
	import { GROUP_ROLE_HELP, GROUP_ROLES, ROLE_LABELS } from '#lib/roles.js';
	import Button from '#lib/ui/Button.svelte';
	import CheckboxGroup from '#lib/ui/CheckboxGroup.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import Tag from '#lib/ui/Tag.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);
	// Save and Cancel go back to where the user came from.
	const previous = back(() => resolve('/groups'));

	const group = $derived(data.group);
	const message = $derived(form && 'message' in form ? form.message : undefined);
	const rolesMessage = $derived(form && 'rolesMessage' in form ? form.rolesMessage : undefined);
	const removeMessage = $derived(form && 'removeMessage' in form ? form.removeMessage : undefined);

	type Member = (typeof data.group.member_list)[number];

	const columns: TableColumn<Member>[] = [
		{ key: 'name', header: 'Name' },
		{ key: 'roles', header: 'Roles', wrap: true },
		{ key: 'email', header: 'Login', wideOnly: true }
	];

	// Pilot first, so who flies is in the same place on every row.
	const ordered = (roles: Member['roles']) => [
		...roles.filter((role) => role === 'pilot'),
		...roles.filter((role) => role !== 'pilot')
	];

	const roleOptions = GROUP_ROLES.map((role) => ({
		value: role,
		label: ROLE_LABELS[role],
		helper: GROUP_ROLE_HELP[role]
	}));

	// What it holds, as "340 flights, 12 sessions and 3 drones".
	const holds = $derived(
		(
			[
				[group.flights, 'flight'],
				[group.sessions, 'session'],
				[group.drones, 'drone'],
				[group.controllers, 'controller']
			] as const
		)
			.filter(([count]) => count > 0)
			.map(([count, noun]) => `${count.toLocaleString('en-US')} ${noun}${count === 1 ? '' : 's'}`)
			.join(', ')
			.replace(/, ([^,]*)$/, ' and $1')
	);
	const ORGANISATION = 'organisation';
	let moveTo = $state('');
	const moveError = $derived(form && 'moveError' in form ? form.moveError : undefined);
	const deleteMessage = $derived(form && 'deleteMessage' in form ? form.deleteMessage : undefined);
	const destination = $derived(
		moveTo === ORGANISATION
			? 'the organisation'
			: data.others.find((other) => other.id === moveTo)?.name
	);

	const members = $derived(group.member_list.length);
	const pilots = $derived(
		group.member_list.filter((member) => member.roles.includes('pilot')).length
	);
	const summary = $derived(
		`${members} ${members === 1 ? 'member' : 'members'}, ` +
			`${pilots} ${pilots === 1 ? 'pilot' : 'pilots'}`
	);
</script>

<PageHeader title={group.name} breadcrumb={[{ href: resolve('/groups'), label: 'Groups' }]}>
	{#snippet actions()}
		<!-- One action; the dialog asks where everything goes when the group
		     isn't empty (carbon-expert's advice). -->
		<Modal
			title="Delete group"
			label={group.name}
			triggerKind="danger-tertiary"
			submitLabel="Delete group"
			danger
			action="?/delete"
			error={deleteMessage}
			onsuccess={() =>
				toast(
					holds
						? `Group ${group.name} deleted. Its contents are now in ${destination}.`
						: `Group ${group.name} deleted.`
				)}
		>
			{#if holds}
				<p>
					{group.name} holds {holds}. Choose where to move them. Then the group is deleted, and its
					members lose their roles in it. The group and its memberships can’t be restored.
				</p>
				<Select
					label="Move contents to"
					name="move_to"
					placeholder="Choose a destination"
					options={[
						...data.others.map((other) => ({ value: other.id, label: other.name })),
						{ value: ORGANISATION, label: 'Organisation, no group' }
					]}
					bind:value={moveTo}
					error={moveError}
					required
					helper={moveTo === ORGANISATION
						? 'Flights wait for a group in the queue; sessions, drones and controllers belong to the organisation without a group.'
						: undefined}
				/>
			{:else}
				<p>
					{group.name} holds no flights, sessions, drones or controllers. Its members lose their roles
					in it. This can’t be undone.
				</p>
			{/if}
		</Modal>
	{/snippet}
</PageHeader>

{#snippet addMember()}
	<Modal
		title="Add member"
		submitLabel="Add"
		action="?/roles"
		error={rolesMessage}
		onsuccess={(added) => toast(`${added?.member} added.`)}
	>
		<Select
			label="User"
			name="user_id"
			placeholder="Choose a user"
			options={data.candidates.map((user) => ({ value: user.id, label: user.name }))}
			required
		/>
		<CheckboxGroup legend="Roles" name="role" options={roleOptions} value={[]} />
	</Modal>
{/snippet}

<Grid rowGap>
	{#if message}
		<Column narrow lg={8}>
			<InlineNotification kind="error" title={message} />
		</Column>
	{/if}
	<Column lg={8}>
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
					toast('Group saved.');
				};
			}}
		>
			<TextInput label="Name" name="name" value={group.name} required autocomplete="off" />
			<div class="buttons">
				<Button type="submit" disabled={busy} fill>Save</Button>
				<Button kind="secondary" href={previous.href} onclick={previous.onclick} fill>Cancel</Button
				>
			</div>
		</form>
	</Column>
	<Column narrow>
		<h2 class="section">Members</h2>
		{#if group.member_list.length === 0}
			<EmptyState
				title="Add the first member"
				body="Members see or organise this group’s flights, sessions and equipment by their roles; pilots fly for it."
				action={data.candidates.length > 0 ? addMember : undefined}
			/>
		{:else}
			<DataTable
				label="Members"
				{columns}
				rows={group.member_list}
				rowKey={(member) => member.user_id}
				rowHref={(member) => resolve('/users/[id]', { id: member.user_id })}
				{summary}
			>
				{#snippet toolbar()}
					<!-- Disabled, not hidden, when everyone is a member: that changes
					     with a new user (Carbon's disabled states); the note says so. -->
					{#if data.candidates.length > 0}
						{@render addMember()}
					{:else}
						<Button disabled>Add member</Button>
					{/if}
				{/snippet}
				{#snippet cell(member, column)}
					{#if column.key === 'roles'}
						<span class="tags">
							{#each ordered(member.roles) as role (role)}
								<Tag type={role === 'pilot' ? 'blue' : 'gray'}>{ROLE_LABELS[role]}</Tag>
							{/each}
						</span>
					{:else if column.key === 'email'}
						{member.email ?? 'No login'}
					{:else}
						{member.name}
					{/if}
				{/snippet}
				{#snippet rowAction(member)}
					<span class="actions">
						<Modal
							title="Edit roles"
							triggerIcon={Edit}
							triggerSize="md"
							triggerLabel="Edit roles of {member.name}"
							label={member.name}
							submitLabel="Save"
							action="?/roles"
							error={rolesMessage}
							onsuccess={() => toast(`Roles of ${member.name} saved.`)}
						>
							<input type="hidden" name="user_id" value={member.user_id} />
							<CheckboxGroup
								legend="Roles"
								name="role"
								options={roleOptions}
								value={member.roles}
							/>
						</Modal>
						<Modal
							title="Remove member"
							label={member.name}
							triggerIcon={SubtractAlt}
							triggerSize="md"
							triggerLabel="Remove {member.name} from the group"
							submitLabel="Remove member"
							danger
							action="?/removeMember"
							error={removeMessage}
							onsuccess={() => toast(`${member.name} removed.`)}
						>
							<input type="hidden" name="user_id" value={member.user_id} />
							<p>
								{member.name} loses their roles in {group.name}. Their flights stay as they are. You
								can add them again later.
							</p>
						</Modal>
					</span>
				{/snippet}
			</DataTable>
			{#if data.candidates.length === 0}
				<p class="note">Every user is a member already. New users are created on the Users page.</p>
			{/if}
		{/if}
	</Column>
</Grid>

<style>
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
		margin-bottom: var(--cds-spacing-05);
		/* The column has no gutter; line up with the table's text. */
		padding-inline: var(--cds-spacing-05);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}

	/* Tags 8px apart, wrapping on narrow screens. */
	.tags {
		display: inline-flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-03);
	}

	.actions {
		display: inline-flex;
	}

	.note {
		margin: var(--cds-spacing-05) 0 0;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
	}
</style>
