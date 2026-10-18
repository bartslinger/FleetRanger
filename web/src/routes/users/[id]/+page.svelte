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
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	const user = $derived(data.user);
	const disabled = $derived(user.disabled_at !== null);
	// Admins can't change their own role or disable themselves; the backend
	// refuses it too.
	const self = $derived(data.account?.email === user.email);
	const message = $derived(form && 'message' in form ? form.message : undefined);

	// svelte-ignore state_referenced_locally
	let role: Role = $state(data.user.role);
</script>

<PageHeader
	title={user.name ?? user.email}
	breadcrumb={[{ href: resolve('/users'), label: 'Users' }]}
	status={disabled ? 'Disabled' : undefined}
>
	{#snippet actions()}
		<!-- Apart from the form below: these apply at once. -->
		{#if disabled}
			<form
				method="POST"
				action="?/enable"
				use:enhance={() =>
					async ({ result, update }) => {
						await update();
						if (result.type === 'success') toast('User enabled.');
					}}
			>
				<Button type="submit" kind="tertiary">Enable user</Button>
			</form>
		{:else if !self}
			<Modal
				title="Disable user"
				triggerKind="tertiary"
				submitLabel="Disable user"
				action="?/disable"
				error={form && 'disableMessage' in form ? form.disableMessage : undefined}
				onsuccess={() => toast('User disabled.')}
			>
				<p>
					{user.email} is logged out everywhere and can't log in until you enable them again.
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
		<!-- Read-only facts: from the login provider and the pilot's page. -->
		<dl class="facts">
			<div>
				<dt>Email</dt>
				<dd>{user.email}</dd>
			</div>
			<div>
				<dt>Name</dt>
				<dd>{user.name ?? 'Shown after the first login'}</dd>
			</div>
			<div>
				<dt>Pilot</dt>
				<dd>
					{#if user.pilot_id}
						<a href={resolve('/pilots/[id]', { id: user.pilot_id })}>{user.pilot_name}</a>
					{:else}
						None. Link a pilot to this user on the pilot's page.
					{/if}
				</dd>
			</div>
			{#if self}
				<div>
					<dt>Role</dt>
					<dd>{ROLE_LABELS[user.role]}. You can’t change your own role.</dd>
				</div>
			{/if}
			<div>
				<dt>Last login</dt>
				<dd>
					{user.last_login_at ? dateTime(user.last_login_at, data.timeZone) : 'Never'}
				</dd>
			</div>
		</dl>
		{#if !self}
			<form
				class="form"
				method="POST"
				action="?/save"
				use:enhance={() => {
					busy = true;
					return async ({ result, update }) => {
						busy = false;
						await update({ reset: false });
						if (result.type === 'success') toast('User saved.');
					};
				}}
			>
				<div>
					<RadioButtonGroup legend="Role" name="role" options={ROLE_OPTIONS} bind:value={role} />
					<p class="help">{ROLE_HELP}</p>
				</div>
				<div class="buttons">
					<Button type="submit" disabled={busy}>Save</Button>
					<Button kind="secondary" href={resolve('/users')}>Cancel</Button>
				</div>
			</form>
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

	a {
		color: var(--cds-link-primary);
	}

	a:hover {
		color: var(--cds-link-primary-hover);
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
