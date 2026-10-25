<script lang="ts">
	import { resolve } from '$app/paths';
	import CodeSnippet from '#lib/ui/CodeSnippet.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { dateTime } from '#lib/format.js';
	import AppDialog from '../AppDialog.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const app = $derived(data.app);
	const message = $derived(form && 'message' in form ? form.message : undefined);
	const deleteMessage = $derived(form && 'deleteMessage' in form ? form.deleteMessage : undefined);
	const people = $derived(`${app.users} ${app.users === 1 ? 'person' : 'people'}`);
</script>

<PageHeader title={app.name} breadcrumb={[{ href: resolve('/apps'), label: 'Apps' }]}>
	{#snippet actions()}
		{#key app}
			<AppDialog
				title="Edit app"
				triggerKind="tertiary"
				submitLabel="Save"
				action="?/save"
				error={message}
				{app}
			/>
		{/key}
		<Modal
			title="Delete app"
			label={app.name}
			triggerKind="danger-tertiary"
			submitLabel="Delete app"
			danger
			action="?/delete"
			error={deleteMessage}
			onsuccess={() => toast(`App ${app.name} deleted.`)}
		>
			<p>
				{app.users > 0
					? `${people} use ${app.name}. Their access ends at once: the app can’t read anything from FleetRanger for them anymore.`
					: `${app.name} can’t log anyone in anymore.`}
				Adding it again gives it a new client ID, to set in the app again.
			</p>
		</Modal>
	{/snippet}
</PageHeader>

<Grid>
	<Column lg={8}>
		<dl class="facts">
			<!-- First: the one fact to act on (carbon-expert's advice). -->
			<div>
				<dt>Client ID</dt>
				<dd class="snippet">
					<CodeSnippet code={app.id} label="client ID" />
					<span class="helper">Copy this into the app’s settings.</span>
				</dd>
			</div>
			<div>
				<dt>Return addresses</dt>
				<dd>
					<ul>
						{#each app.redirect_uris as uri (uri)}<li>{uri}</li>{/each}
					</ul>
				</dd>
			</div>
			<div>
				<dt>Used by</dt>
				<dd>{app.users > 0 ? people : 'Nobody yet'}</dd>
			</div>
			<div>
				<dt>Added</dt>
				<dd>{dateTime(app.created_at, data.timeZone)}</dd>
			</div>
		</dl>
	</Column>
</Grid>

<style>
	.facts {
		display: grid;
		gap: var(--cds-spacing-05);
		margin: 0 0 var(--cds-spacing-07);
	}

	dt,
	.helper {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.helper {
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		overflow-wrap: anywhere;
	}

	.snippet {
		display: grid;
		/* Lets the snippet narrow, scrolling the ID, instead of running off phones. */
		grid-template-columns: minmax(0, 1fr);
		gap: var(--cds-spacing-02);
		margin-top: var(--cds-spacing-03);
	}

	ul {
		margin: 0;
		padding: 0;
		list-style: none;
	}
</style>
