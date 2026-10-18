<script lang="ts">
	import { enhance } from '$app/forms';
	import { resolve } from '$app/paths';
	import { dateTime } from '#lib/format.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let busy = $state(false);
	const paired = $derived(form && 'paired' in form ? form.paired : undefined);
	const message = $derived(form && 'message' in form ? form.message : undefined);
</script>

<PageHeader
	title="Pair a controller"
	description="Link a remote controller to FleetRanger, so it can offer your pilots and upload its flight logs."
/>

<Grid rowGap>
	{#if paired}
		<Column narrow lg={8}>
			<InlineNotification kind="success" title="Controller paired">
				The controller ({paired}) is ready. It continues by itself.
			</InlineNotification>
		</Column>
	{:else if data.request}
		{#if message}
			<Column narrow lg={8}>
				<InlineNotification kind="error" title={message} />
			</Column>
		{/if}
		<Column lg={8}>
			<dl class="facts">
				<div>
					<dt>Controller</dt>
					<dd>{data.request.model}</dd>
				</div>
				<div>
					<dt>Serial number</dt>
					<dd class="identifier">{data.request.serial}</dd>
				</div>
				<div>
					<dt>Team</dt>
					<dd>{data.request.team_name ?? 'None. An admin can set it later.'}</dd>
				</div>
				<div>
					<dt>Code valid until</dt>
					<dd>{dateTime(data.request.expires_at, data.timeZone)} ({data.timeZoneName})</dd>
				</div>
			</dl>
			<p class="check">Check that the serial number matches the one on the controller’s screen.</p>
			<form
				method="POST"
				action="?/approve"
				use:enhance={() => {
					busy = true;
					return async ({ update }) => {
						busy = false;
						await update();
					};
				}}
			>
				<input type="hidden" name="code" value={data.code} />
				<div class="buttons">
					<Button type="submit" disabled={busy}>Pair controller</Button>
					<Button kind="secondary" href={resolve('/')}>Cancel</Button>
				</div>
			</form>
		</Column>
	{:else}
		{#if data.code}
			<Column narrow lg={8}>
				<InlineNotification kind="error" title="No controller is waiting with this code">
					Codes expire after 10 minutes. Start pairing on the controller again, and enter the new
					code.
				</InlineNotification>
			</Column>
		{/if}
		<Column lg={8}>
			<!-- A GET form: the code goes into the URL, as from the controller's QR code. -->
			<form class="form" method="GET">
				<TextInput
					label="Code"
					name="code"
					helper="Shown on the controller, e.g. KDW4-9TXR"
					autocomplete="off"
					autocapitalize="characters"
					spellcheck="false"
					required
				/>
				<div class="buttons">
					<Button type="submit">Continue</Button>
				</div>
			</form>
		</Column>
	{/if}
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

	/* Related facts 16px apart, as on the flight page. */
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0;
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

	.check {
		margin: var(--cds-spacing-07) 0 var(--cds-spacing-05);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}
</style>
