<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { KINDS } from '#lib/equipment.js';
	import { dateTime } from '#lib/format.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);

	const item = $derived(data.item);
	const kind = $derived(KINDS[data.kind]);
	const list = $derived(
		resolve(data.kind === 'drones' ? '/equipment' : '/equipment?tab=controllers')
	);
	const message = $derived(form && 'message' in form ? form.message : undefined);

	// Active teams and users, plus the current ones if archived since.
	const teamOptions = $derived([
		{ value: '', label: 'None' },
		...data.teams.map((team) => ({ value: team.id, label: team.name })),
		...(item.team_id && !data.teams.some((team) => team.id === item.team_id)
			? [{ value: item.team_id, label: `${item.team_name} (archived)` }]
			: [])
	]);
	const holderOptions = $derived([
		{ value: '', label: 'None' },
		...data.users.map((user) => ({ value: user.id, label: user.name })),
		...(item.holder_user_id && !data.users.some((user) => user.id === item.holder_user_id)
			? [{ value: item.holder_user_id, label: `${item.holder_name} (archived)` }]
			: [])
	]);
</script>

<PageHeader title={item.name ?? item.serial} breadcrumb={[{ href: list, label: 'Equipment' }]} />

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
			<div>
				<dt>Last flight</dt>
				<dd>
					{item.last_flight_at
						? `${dateTime(item.last_flight_at, data.timeZone)} (${data.timeZoneName})`
						: 'None'}
				</dd>
			</div>
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
					if (result.type === 'success') toast(`${kind.one} saved.`);
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
			<Select
				label="Team"
				name="team_id"
				options={teamOptions}
				value={item.team_id ?? ''}
				helper={data.kind === 'controllers'
					? 'New flights from this controller without a pilot go to this team.'
					: undefined}
			/>
			<Select
				label="Holder"
				name="holder_user_id"
				options={holderOptions}
				value={item.holder_user_id ?? ''}
				helper="Who has it now. For information only; it gives no access."
			/>
			<div class="buttons">
				<Button type="submit" disabled={busy}>Save</Button>
				<Button kind="secondary" href={list}>Cancel</Button>
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
