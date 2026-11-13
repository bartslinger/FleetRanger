<script lang="ts">
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import NumberInput from '#lib/ui/NumberInput.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const organisation = $derived(data.organisation);
	const message = $derived(form && 'message' in form ? form.message : undefined);
	const deleteMessage = $derived(form && 'deleteMessage' in form ? form.deleteMessage : undefined);
	// Reset to the saved value each time the dialog's fields are drawn anew.
	let gap = $derived(organisation.session_gap_minutes);

	/** E.g. "2 hours", "90 minutes". */
	function duration(minutes: number): string {
		if (minutes % 60 === 0) {
			const hours = minutes / 60;
			return `${hours} ${hours === 1 ? 'hour' : 'hours'}`;
		}
		return `${minutes} ${minutes === 1 ? 'minute' : 'minutes'}`;
	}
</script>

<Grid>
	<Column lg={8}>
		<!-- In the tab, not the page header, which both tabs share (carbon-expert's
		     advice, user's choice 2026-11-13). -->
		<div class="toolbar">
			{#key organisation}
				<Modal
					title="Edit settings"
					triggerKind="tertiary"
					label={organisation.name}
					submitLabel="Save"
					action="?/save"
					error={message}
					onsuccess={() => toast('Settings saved.')}
				>
					<TextInput
						label="Name"
						name="name"
						value={organisation.name}
						required
						autocomplete="off"
					/>
					<Select
						label="Time zone"
						name="time_zone"
						helper="All times are shown in it."
						options={organisation.time_zones.map((zone) => ({ value: zone, label: zone }))}
						value={organisation.time_zone}
						required
					/>
					<NumberInput
						label="Session gap"
						name="session_gap_minutes"
						helper="Minutes. A longer break between flights starts a new session."
						bind:value={gap}
						min={1}
						required
					/>
					{#if organisation.mcp_offered}
						<input type="hidden" name="mcp_offered" value="on" />
						<!-- A checkbox, not a toggle: it applies only with Save (Carbon's
						     toggle guidance). -->
						<Checkbox
							label="AI assistants can query FleetRanger"
							name="mcp"
							checked={organisation.mcp}
						/>
					{/if}
				</Modal>
			{/key}
		</div>
		<dl class="settings" aria-label="Organisation settings">
			<div>
				<dt>Name</dt>
				<dd>{organisation.name}</dd>
			</div>
			<div>
				<dt>Time zone</dt>
				<dd>{organisation.time_zone}</dd>
			</div>
			<div>
				<dt>Session gap</dt>
				<dd>{duration(organisation.session_gap_minutes)}</dd>
			</div>
			{#if organisation.mcp_offered}
				<div>
					<dt>AI assistants</dt>
					<dd>{organisation.mcp ? 'On' : 'Off'}</dd>
				</div>
			{/if}
		</dl>
		<!-- Carbon's high-impact deletion: a danger modal where the name is typed
		     (Common actions, "Delete"); a danger tertiary trigger as a side action,
		     in its own block away from Edit settings (carbon-expert's advice). -->
		<section class="delete" aria-labelledby="delete-heading">
			<h2 id="delete-heading">Delete organisation</h2>
			<p>Deletes the organisation and all its data at once.</p>
			<Modal
				title="Delete organisation"
				triggerKind="danger-tertiary"
				label={organisation.name}
				submitLabel="Delete organisation"
				danger
				action="?/delete"
				error={deleteMessage}
			>
				<p>
					This deletes all flights, sessions, equipment, users and raw logs of {organisation.name}.
					It can't be undone.
				</p>
				<TextInput
					label="Type {organisation.name} to confirm"
					name="name"
					required
					autocomplete="off"
				/>
			</Modal>
		</section>
	</Column>
</Grid>

<style>
	/* The action above the list on the right, where the Apps tab has its own
	   (carbon-expert's advice). */
	.toolbar {
		display: flex;
		justify-content: flex-end;
		margin-bottom: var(--cds-spacing-05);
	}

	/* Carbon's structured list, read-only and flush: transparent on the page,
	   a subtle divider under each row (no header row), 16px above and 24px below the text,
	   none on the left so the text keeps the page's key line. */
	.settings {
		margin: 0;
	}

	.settings div {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 2fr);
		gap: var(--cds-spacing-05);
		padding: var(--cds-spacing-05) var(--cds-spacing-05) var(--cds-spacing-06) 0;
		border-bottom: 1px solid var(--cds-border-subtle-00);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	dt {
		color: var(--cds-text-primary);
		font-weight: var(--cds-heading-compact-01-font-weight);
	}

	/* Its own block, a section's spacing below the settings. */
	.delete {
		margin-top: var(--cds-spacing-09);
	}

	.delete h2 {
		margin: 0 0 var(--cds-spacing-03);
		font-size: var(--cds-heading-compact-02-font-size);
		font-weight: var(--cds-heading-compact-02-font-weight);
		line-height: var(--cds-heading-compact-02-line-height);
		letter-spacing: var(--cds-heading-compact-02-letter-spacing);
	}

	.delete > p {
		margin: 0 0 var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
	}

	dd {
		margin: 0;
		color: var(--cds-text-primary);
		overflow-wrap: anywhere;
	}
</style>
