<script lang="ts">
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { back } from '#lib/back.svelte.js';
	import { KINDS } from '#lib/equipment.js';
	import { dateTime } from '#lib/format.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Link from '#lib/ui/Link.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { filtered } from '../../../logbook.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	const item = $derived(data.item);
	const kind = $derived(KINDS[data.kind]);
	const list = $derived(
		resolve(data.kind === 'drones' ? '/equipment' : '/equipment?tab=controllers')
	);
	const message = $derived(form && 'message' in form ? form.message : undefined);
	// Save and Cancel go back to where the user came from, e.g. a flight's page.
	const previous = back(() => list);

	// Active teams and holders, plus the current ones if archived since. Members
	// choose from their own team; admins from everyone.
	const teamOptions = $derived([
		{ value: '', label: 'None' },
		...data.teams.map((team) => ({ value: team.id, label: team.name })),
		...(item.team_id && !data.teams.some((team) => team.id === item.team_id)
			? [{ value: item.team_id, label: `${item.team_name} (archived)` }]
			: [])
	]);
	const holderOptions = $derived([
		{ value: '', label: 'None' },
		...data.holders.map((user) => ({ value: user.id, label: user.name })),
		...(item.holder_user_id && !data.holders.some((user) => user.id === item.holder_user_id)
			? [{ value: item.holder_user_id, label: `${item.holder_name} (archived)` }]
			: [])
	]);
	const flights = $derived(
		`${resolve('/')}${filtered(data.kind === 'drones' ? { drone: item.serial } : { controller: item.serial })}`
	);
</script>

<PageHeader title={item.name ?? item.serial} breadcrumb={[{ href: list, label: 'Equipment' }]}>
	{#snippet actions()}
		{#if item.paired && data.edit === 'all'}
			<Modal
				title="Remove pairing"
				triggerKind="danger-tertiary"
				submitLabel="Remove pairing"
				danger
				action="?/unpair"
				error={form && 'unpairMessage' in form ? form.unpairMessage : undefined}
				onsuccess={() => toast('Pairing removed.')}
			>
				<p>
					The controller stops uploading flight logs and can’t download the pilot list. Its flights
					stay as they are. To use it again, pair it again from the controller.
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
		<!-- Read-only facts, from the flight logs. -->
		<dl class="facts">
			<div>
				<dt>Kind</dt>
				<dd>{kind.one}</dd>
			</div>
			<div>
				<dt>Serial number</dt>
				<dd class="identifier">{item.serial}</dd>
			</div>
			{#if item.paired !== null}
				<div>
					<dt>Pairing</dt>
					<dd>{item.paired ? 'Paired' : 'Not paired'}</dd>
				</div>
				<!-- The app syncs on connections its settings allow, also without new logs. -->
				<div>
					<dt>Last sync</dt>
					<dd>
						{item.synced_at
							? `${dateTime(item.synced_at, data.timeZone)} (${data.timeZoneName})`
							: 'Never'}
					</dd>
				</div>
				{#if item.app_version}
					<div>
						<dt>App version</dt>
						<dd>{item.app_version}</dd>
					</div>
				{/if}
			{/if}
			<div>
				<dt>Last flight</dt>
				<dd>
					{item.last_flight_at
						? `${dateTime(item.last_flight_at, data.timeZone)} (${data.timeZoneName})`
						: 'None'}
				</dd>
			</div>
			<!-- Who can't change them sees them here; members change only the holder. -->
			{#if data.edit !== 'all'}
				<div>
					<dt>Team</dt>
					<dd>{item.team_name ?? 'None'}</dd>
				</div>
			{/if}
			{#if data.edit === 'none'}
				<div>
					<dt>Holder</dt>
					<dd>{item.holder_name ?? 'None'}</dd>
				</div>
			{/if}
		</dl>
		<p class="flights">
			<Link href={flights}>
				{data.kind === 'drones' ? 'Flights with this drone' : 'Flights with this controller'}
			</Link>
		</p>
		{#if data.edit !== 'none'}
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
						toast(`${kind.one} saved.`);
					};
				}}
			>
				<TextInput
					label="Name (optional)"
					name="name"
					value={item.name ?? ''}
					autocomplete="off"
					helper="For example the number painted on it."
				/>
				{#if data.edit === 'all'}
					<Select
						label="Team"
						name="team_id"
						options={teamOptions}
						value={item.team_id ?? ''}
						helper={data.kind === 'controllers'
							? 'New flights from this controller without a pilot go to this team.'
							: undefined}
					/>
				{:else}
					<!-- Unchanged: only admins move equipment between teams. -->
					<input type="hidden" name="team_id" value={item.team_id ?? ''} />
				{/if}
				<Select
					label="Holder"
					name="holder_user_id"
					options={holderOptions}
					value={item.holder_user_id ?? ''}
					helper="Who has it now. For information only; it gives no access."
				/>
				<div class="buttons">
					<Button type="submit" disabled={busy}>Save</Button>
					<Button kind="secondary" href={previous.href}>Cancel</Button>
				</div>
			</form>
		{/if}
	</Column>
</Grid>

<style>
	/* The link 16px after the facts, as between them; 32px before the form. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-05);
	}

	.flights {
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

	.identifier {
		font-variant-numeric: tabular-nums slashed-zero;
	}

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
