<script lang="ts">
	import SubtractAlt from '@carbon/icons/es/subtract--alt/16.js';
	import { enhance } from '$app/forms';
	import { dateTime } from '#lib/format.js';
	import type { PairingCode } from '#lib/server/backend.js';
	import Button from '#lib/ui/Button.svelte';
	import CodeSnippet from '#lib/ui/CodeSnippet.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);
	const created = $derived(form && 'created' in form ? form.created : undefined);
	const message = $derived(form && 'message' in form ? form.message : undefined);
	const revokeMessage = $derived(form && 'revokeMessage' in form ? form.revokeMessage : undefined);

	/** KDW49TXR as KDW4-9TXR, easier to read out and type; the controller takes either. */
	const shown = (code: string) => `${code.slice(0, 4)}-${code.slice(4)}`;
	const until = (iso: string) => dateTime(iso, data.timeZone);

	// Admins see everyone's codes, so who made each. On phones only the code
	// and its time, so Revoke stays in view.
	const columns: TableColumn<PairingCode>[] = $derived([
		{ key: 'code', header: 'Code', identifier: true, format: (row) => shown(row.code) },
		{ key: 'team', header: 'Team', wideOnly: true, format: (row) => row.team ?? 'No team' },
		...(data.account?.role === 'admin'
			? [
					{
						key: 'created_by',
						header: 'Created by',
						wideOnly: true
					} satisfies TableColumn<PairingCode>
				]
			: []),
		{ key: 'expires_at', header: 'Valid until', format: (row) => until(row.expires_at) }
	]);

	const validity = [
		{ value: '1', label: '1 hour' },
		{ value: '24', label: '24 hours' },
		{ value: '72', label: '3 days' },
		{ value: '168', label: '7 days' }
	];
</script>

<PageHeader
	title="Pair a controller"
	description="Create a code for the pilot to type into the controller. It pairs one controller, then stops working."
/>

<Grid>
	<Column lg={8}>
		<form
			class="form"
			method="POST"
			action="?/create"
			use:enhance={() => {
				busy = true;
				return async ({ update }) => {
					busy = false;
					await update({ reset: false });
				};
			}}
		>
			<!-- The controller joins the creator's team: which one, if they're in several. -->
			{#if data.teams.length > 1}
				<Select
					label="Team"
					name="team_id"
					options={data.teams.map((team) => ({ value: team.id, label: team.name }))}
					helper="The controller joins this team. Its flights without a chosen team go to it."
				/>
			{/if}
			<Select
				label="Valid for"
				name="hours"
				value="24"
				options={validity}
				helper="After this, an unused code stops working."
			/>
			{#if message}
				<InlineNotification kind="error" title={message} />
			{/if}
			<div class="buttons">
				<Button type="submit" disabled={busy}>Create code</Button>
			</div>
		</form>

		{#if created}
			<div class="created">
				<InlineNotification kind="success" title="Code created">
					<p>
						{created.team
							? `The controller joins ${created.team}.`
							: 'The controller joins no team.'}
						Valid until {until(created.expires_at)}.
					</p>
				</InlineNotification>
				<CodeSnippet code={shown(created.code)} label="pairing code" />
			</div>
		{/if}

		<h2 class="heading">Open codes</h2>
		{#if data.codes.length === 0}
			<EmptyState
				title="No open codes"
				body="Codes you create appear here until they’re used or expire."
			/>
		{:else}
			<DataTable label="Open pairing codes" {columns} rows={data.codes} rowKey={(row) => row.id}>
				{#snippet rowAction(row)}
					<!-- Confirmed first (user's request), though a new code is quick to make. -->
					<Modal
						title="Revoke code"
						label="Code {shown(row.code)}"
						triggerIcon={SubtractAlt}
						triggerLabel="Revoke code {shown(row.code)}"
						triggerSize="md"
						submitLabel="Revoke code"
						danger
						action="?/revoke"
						error={revokeMessage}
						onsuccess={() => toast(`Code ${shown(row.code)} revoked.`)}
					>
						<input type="hidden" name="id" value={row.id} />
						<p>
							The code stops working, also for a pilot who already has it. A controller it paired
							stays paired.
						</p>
					</Modal>
				{/snippet}
			</DataTable>
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

	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-05);
	}

	.created {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-07);
	}

	.created p {
		margin: 0;
	}

	.heading {
		margin: var(--cds-spacing-09) 0 var(--cds-spacing-05);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}
</style>
