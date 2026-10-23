<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { back } from '#lib/back.svelte.js';
	import { KINDS, lastPilot } from '#lib/equipment.js';
	import { dateTime, duration } from '#lib/format.js';
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
	import { filtered } from '../../../flights/logbook.js';
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

	// Where the user may move it, and where it is now.
	const groupOptions = $derived([
		...(data.toOrganisation || item.group_id === null
			? [{ value: '', label: 'Whole organisation' }]
			: []),
		...data.groups.map((group) => ({ value: group.id, label: group.name })),
		...(item.group_id && !data.groups.some((group) => group.id === item.group_id)
			? [{ value: item.group_id, label: item.group_name ?? '' }]
			: [])
	]);
	const flights = $derived(
		`${resolve('/flights')}${filtered(data.kind === 'drones' ? { drone: item.serial } : { controller: item.serial })}`
	);
</script>

<PageHeader title={item.name ?? item.serial} breadcrumb={[{ href: list, label: 'Equipment' }]}>
	{#snippet actions()}
		{#if item.paired && data.unpair}
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
					stay as they are. To use it again, pair it again with a new pairing code.
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
						{item.synced_at ? dateTime(item.synced_at, data.timeZone) : 'Never'}
					</dd>
				</div>
				{#if item.app_version}
					<div>
						<dt>App version</dt>
						<dd>{item.app_version}</dd>
					</div>
				{/if}
			{/if}
			<!-- Over all its flights, also those the user doesn't see. -->
			<div>
				<dt>Flights</dt>
				<dd>{item.flights.toLocaleString('en-US')}</dd>
			</div>
			<div>
				<dt>Air time</dt>
				<dd>{duration(item.air_time_s)}</dd>
			</div>
			<div>
				<dt>Last flight</dt>
				<dd>
					{item.last_flight_at ? dateTime(item.last_flight_at, data.timeZone) : 'None'}
				</dd>
			</div>
			{#if item.last_flight_at}
				<div>
					<dt>Last flown by</dt>
					<dd>{lastPilot(item)}</dd>
				</div>
			{/if}
			<!-- Who can't change it sees it here. -->
			{#if !data.move}
				<div>
					<dt>Group</dt>
					<dd>{item.group_name ?? 'Whole organisation'}</dd>
				</div>
			{/if}
		</dl>
		<p class="flights">
			<Link href={flights}>
				{data.kind === 'drones' ? 'Flights with this drone' : 'Flights with this controller'}
			</Link>
		</p>
		{#if data.rename || data.move}
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
						toast(`${kind.one} saved.`);
					};
				}}
			>
				{#if data.rename}
					<TextInput
						label="Name (optional)"
						name="name"
						value={item.name ?? ''}
						autocomplete="off"
						helper="For example the number painted on it."
					/>
				{:else}
					<!-- Unchanged: renaming needs equipment.rename on its group. -->
					<input type="hidden" name="name" value={item.name ?? ''} />
				{/if}
				{#if data.move}
					<Select
						label="Group"
						name="group_id"
						options={groupOptions}
						value={item.group_id ?? ''}
					/>
				{:else}
					<!-- Unchanged: moving needs equipment.move on both groups. -->
					<input type="hidden" name="group_id" value={item.group_id ?? ''} />
				{/if}
				<div class="buttons">
					<Button type="submit" disabled={busy} fill>Save</Button>
					<Button kind="secondary" href={previous.href} onclick={previous.onclick} fill
						>Cancel</Button
					>
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

	/* Carbon's fluid pair: equal halves of the fields' width, 1px apart. */
	.buttons {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		margin-top: var(--cds-spacing-05);
	}
</style>
