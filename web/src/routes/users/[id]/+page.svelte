<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { dateTime } from '#lib/format.js';
	import { ROLE_HELP, ROLE_LABELS, ROLE_OPTIONS, type Role } from '#lib/roles.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import RadioButtonGroup from '#lib/ui/RadioButtonGroup.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	const user = $derived(data.user);
	const archived = $derived(user.archived_at !== null);
	// Admins can't change their own role, remove their own login, or archive or
	// delete themselves; the backend refuses it too.
	const self = $derived(data.account?.id === user.id);
	const message = $derived(form && 'message' in form ? form.message : undefined);

	// svelte-ignore state_referenced_locally
	let role: Role = $state(data.user.role);

	// Active teams, plus the user's own if it has been archived since.
	const teamOptions = $derived([
		{ value: '', label: 'None' },
		...data.teams.map((team) => ({ value: team.id, label: team.name })),
		...(user.team_id && !data.teams.some((team) => team.id === user.team_id)
			? [{ value: user.team_id, label: `${user.team_name} (archived)` }]
			: [])
	]);
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
					{user.last_login_at
						? `${dateTime(user.last_login_at, data.timeZone)} (${data.timeZoneName})`
						: 'Never'}
				</dd>
			</div>
			{#if self}
				<div>
					<dt>Role</dt>
					<dd>{ROLE_LABELS[user.role]}. You can’t change your own role.</dd>
				</div>
			{/if}
		</dl>
		<form
			class="form"
			method="POST"
			action="?/save"
			use:enhance={() => {
				busy = true;
				return async ({ result, update }) => {
					busy = false;
					await update({ reset: false });
					if (result.type === 'success') toast(`${user.name} saved.`);
				};
			}}
		>
			<TextInput label="Name" name="name" value={user.name} required autocomplete="off" />
			<Select
				label="Team"
				name="team_id"
				options={teamOptions}
				value={user.team_id ?? ''}
				helper="New flights go to this team; past flights keep theirs. None for someone who doesn't fly."
			/>
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
			{#if self}
				<input type="hidden" name="role" value={user.role} />
			{:else}
				<div>
					<RadioButtonGroup legend="Role" name="role" options={ROLE_OPTIONS} bind:value={role} />
					<p class="help">{ROLE_HELP}</p>
				</div>
			{/if}
			<div class="buttons">
				<Button type="submit" disabled={busy}>Save</Button>
				<Button kind="secondary" href={resolve('/users')}>Cancel</Button>
			</div>
		</form>
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

	.help {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}

	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-05);
	}
</style>
