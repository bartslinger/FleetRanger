<script lang="ts">
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { back } from '#lib/back.svelte.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
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
	const previous = back(() => resolve('/teams'));

	const archived = $derived(data.team.archived_at !== null);
	const message = $derived(form && 'message' in form ? form.message : undefined);
</script>

<PageHeader
	title={data.team.name}
	breadcrumb={[{ href: resolve('/teams'), label: 'Teams' }]}
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
						if (result.type === 'success') toast('Team restored.');
					}}
			>
				<Button type="submit" kind="tertiary">Restore team</Button>
			</form>
		{:else}
			<Modal
				title="Archive team"
				triggerKind="tertiary"
				submitLabel="Archive team"
				action="?/archive"
				error={form && 'archiveMessage' in form ? form.archiveMessage : undefined}
				onsuccess={() => toast('Team archived.')}
			>
				<p>
					{data.team.name} is hidden from lists and can't get new members. Its flights stay as they are.
					You can restore it later.
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
					if (result.type !== 'success') return update({ reset: false });
					// eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved already
					await goto(previous.href);
					toast('Team saved.');
				};
			}}
		>
			<TextInput label="Name" name="name" value={data.team.name} required autocomplete="off" />
			<div class="buttons">
				<Button type="submit" disabled={busy}>Save</Button>
				<Button kind="secondary" href={previous.href}>Cancel</Button>
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
