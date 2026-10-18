<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	const archived = $derived(data.team.archived_at !== null);
	const message = $derived(form && 'message' in form ? form.message : undefined);
</script>

<PageHeader
	title={data.team.name}
	breadcrumb={[{ href: resolve('/teams'), label: 'Teams' }]}
	status={archived ? 'Archived' : undefined}
>
	{#snippet actions()}
		<!-- Applies at once, apart from the form below. -->
		<form method="POST" action={archived ? '?/restore' : '?/archive'} use:enhance>
			<Button type="submit" kind="tertiary">{archived ? 'Restore team' : 'Archive team'}</Button>
		</form>
	{/snippet}
</PageHeader>

<Grid rowGap>
	{#if message || form}
		<Column narrow lg={8}>
			{#if message}
				<InlineNotification kind="error" title={message} />
			{:else if form && 'saved' in form}
				<InlineNotification kind="success" title="Team saved." />
			{:else if form && 'archived' in form}
				<InlineNotification kind="success" title="Team archived." />
			{:else if form && 'restored' in form}
				<InlineNotification kind="success" title="Team restored." />
			{/if}
		</Column>
	{/if}
	<Column lg={8}>
		<form
			class="form"
			method="POST"
			action="?/save"
			use:enhance={() => {
				busy = true;
				return async ({ update }) => {
					busy = false;
					await update({ reset: false });
				};
			}}
		>
			<TextInput label="Name" name="name" value={data.team.name} required autocomplete="off" />
			<div class="buttons">
				<Button type="submit" disabled={busy}>Save</Button>
				<Button kind="secondary" href={resolve('/teams')}>Cancel</Button>
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
