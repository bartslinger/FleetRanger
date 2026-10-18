<script lang="ts">
	import { enhance } from '$app/forms';
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
	// The code typed, kept after a rejected attempt; else the one from the QR code.
	const code = $derived(form && 'code' in form ? form.code : data.code);
</script>

<PageHeader
	title="Pair a controller"
	description="Link a remote controller to FleetRanger, so it can offer your pilots and upload its flight logs."
/>

<Grid>
	<Column lg={8}>
		{#if paired}
			<InlineNotification kind="success" title="Controller paired">
				{paired.model} ({paired.serial}) continues by itself.
			</InlineNotification>
		{:else}
			{#if data.pairing.length > 0}
				<h2 class="waiting-heading">Waiting to be paired</h2>
				<ul class="waiting">
					{#each data.pairing as controller (controller.serial)}
						<li>
							{controller.model}
							<span class="identifier">{controller.serial}</span>
						</li>
					{/each}
				</ul>
			{/if}
			<form
				class="form"
				method="POST"
				action="?/pair"
				use:enhance={() => {
					busy = true;
					return async ({ update }) => {
						busy = false;
						await update({ reset: false });
					};
				}}
			>
				{#if message}
					<InlineNotification kind="error" title={message} />
				{/if}
				<TextInput
					label="Code"
					name="code"
					value={code}
					helper="Shown on the controller, e.g. KDW4-9TXR"
					autocomplete="off"
					autocapitalize="characters"
					spellcheck="false"
					required
				/>
				<div class="buttons">
					<Button type="submit" disabled={busy}>Pair controller</Button>
				</div>
			</form>
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

	.waiting-heading {
		margin: 0 0 var(--cds-spacing-05);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.waiting {
		display: grid;
		gap: var(--cds-spacing-03);
		margin: 0 0 var(--cds-spacing-07);
		padding: 0;
		list-style: none;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.identifier {
		display: block;
		color: var(--cds-text-secondary);
		font-variant-numeric: tabular-nums slashed-zero;
	}
</style>
