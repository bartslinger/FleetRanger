<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	const pilot = $derived(data.pilot);
	const archived = $derived(pilot.archived_at !== null);
	const message = $derived(form && 'message' in form ? form.message : undefined);

	// Active teams, plus the pilot's own if it has been archived since.
	const teamOptions = $derived([
		...data.teams.map((team) => ({ value: team.id, label: team.name })),
		...(pilot.team_id && !data.teams.some((team) => team.id === pilot.team_id)
			? [{ value: pilot.team_id, label: `${pilot.team_name} (archived)` }]
			: [])
	]);
	const userOptions = $derived([
		{ value: '', label: 'None' },
		...data.users.map((user) => ({
			value: user.id,
			label: user.name ? `${user.name} (${user.email})` : user.email
		}))
	]);
</script>

<PageHeader
	title={pilot.name}
	breadcrumb={[{ href: resolve('/pilots'), label: 'Pilots' }]}
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
						if (result.type === 'success') toast('Pilot restored.');
					}}
			>
				<Button type="submit" kind="tertiary">Restore pilot</Button>
			</form>
		{:else}
			<Modal
				title="Archive pilot"
				triggerKind="tertiary"
				submitLabel="Archive pilot"
				action="?/archive"
				error={form && 'archiveMessage' in form ? form.archiveMessage : undefined}
				onsuccess={() => toast('Pilot archived.')}
			>
				<p>
					{pilot.name} is hidden from lists and can't be chosen as a pilot anymore. Their flights stay
					as they are. You can restore them later.
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
		<form
			class="form"
			method="POST"
			action="?/save"
			use:enhance={() => {
				busy = true;
				return async ({ result, update }) => {
					busy = false;
					await update({ reset: false });
					if (result.type === 'success') toast('Pilot saved.');
				};
			}}
		>
			<TextInput label="Name" name="name" value={pilot.name} required autocomplete="off" />
			<Select
				label="Team"
				name="team_id"
				options={teamOptions}
				value={pilot.team_id ?? undefined}
				helper="New flights go to this team. Past flights keep their team."
				required
			/>
			<Select
				label="Linked user"
				name="account_id"
				options={userOptions}
				value={pilot.account_id ?? ''}
				helper="The user sees this pilot's flights and those of the pilot's team."
			/>
			<div class="buttons">
				<Button type="submit" disabled={busy}>Save</Button>
				<Button kind="secondary" href={resolve('/pilots')}>Cancel</Button>
			</div>
		</form>
	</Column>
</Grid>

<style>
	/* Carbon's in-page form: fields 32px apart, 48px before the buttons. */
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-05);
	}
</style>
