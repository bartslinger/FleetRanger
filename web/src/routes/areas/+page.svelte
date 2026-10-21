<script lang="ts">
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import type { SavedArea } from '#lib/server/backend.js';
	import AreaMap, { type EditState } from '#lib/ui/AreaMap.svelte';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DataTable, { type Column as TableColumn } from '#lib/ui/DataTable.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Link from '#lib/ui/Link.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	type Row = (typeof data.areas)[number];

	const columns: TableColumn<Row>[] = [
		{ key: 'name', header: 'Name' },
		{
			key: 'boundary',
			header: 'Parts',
			numeric: true,
			format: (area) =>
				String(area.boundary.type === 'Polygon' ? 1 : area.boundary.coordinates.length)
		}
	];

	/** The area chosen in the list or on the map, kept in the address. */
	const selected = $derived(
		data.areas.find((area) => area.id === page.url.searchParams.get('area'))
	);

	// The name of the area shown last, kept after it's deleted.
	let shownName = $state('');
	$effect(() => {
		if (selected) shownName = selected.name;
	});

	function areaHref(id: string) {
		return `${resolve('/areas')}?area=${id}`;
	}

	function showArea(id: string, options: { invalidateAll?: boolean } = {}) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- areaHref resolves it
		return goto(areaHref(id), options);
	}

	// One area at a time is added or edited on the map: `new`, an id, or none.
	let editing: string | null = $state(null);
	let edit: EditState = $state({ parts: 0, partSelected: false, drawing: false });
	let map: AreaMap | undefined = $state();
	let busy = $state(false);
	let drawError: string | undefined = $state();
	/** After saving: the areas that clipped the saved one, if any. */
	let clipped: { id: string; by: string[] } | null = $state(null);

	const message = $derived(
		drawError ?? (form && 'message' in form ? (form.message as string) : undefined)
	);
	const nameMessage = $derived(
		form && 'nameMessage' in form ? (form.nameMessage as string) : undefined
	);

	function startEditing(id: string) {
		editing = id;
		drawError = undefined;
		clipped = null;
	}
</script>

<PageHeader title="Areas" />

{#snippet addArea()}
	<span class="wide-only">
		<Button onclick={() => startEditing('new')}>Add area</Button>
	</span>
{/snippet}

<Grid rowGap>
	<Column md={8} lg={4} narrow>
		<div class="panel">
			<div class="phone-only">
				<InlineNotification kind="info" title="Draw and edit areas on a larger screen." />
			</div>
			{#if editing !== null}
				{@const area = data.areas.find((area) => area.id === editing)}
				<form
					class="form inset"
					method="POST"
					action="?/save"
					use:enhance={({ formData, cancel }) => {
						const boundary = map?.boundary();
						if (!boundary) {
							drawError = 'Draw at least one part of the area on the map.';
							cancel();
							return;
						}
						formData.set('boundary', JSON.stringify(boundary));
						drawError = undefined;
						busy = true;
						return async ({ result, update }) => {
							busy = false;
							if (result.type !== 'success') return update({ reset: false });
							const saved = result.data?.saved as SavedArea;
							editing = null;
							clipped = saved.clipped_by.length
								? { id: saved.area.id, by: saved.clipped_by }
								: null;
							await showArea(saved.area.id, { invalidateAll: true });
							toast(`Area ${saved.area.name} saved.`);
						};
					}}
				>
					<h2 class="section">{area ? `Edit ${area.name}` : 'New area'}</h2>
					{#if message}
						<InlineNotification kind="error" title={message} />
					{/if}
					<input type="hidden" name="id" value={area?.id ?? ''} />
					<TextInput
						label="Name"
						name="name"
						value={area?.name ?? ''}
						error={nameMessage}
						required
						autocomplete="off"
					/>
					<!-- Instructions, not a status: plain text (Carbon review). -->
					<p class="instructions" aria-live="polite">
						{#if edit.drawing || (editing === 'new' && edit.parts === 0)}
							Click the map to place corners, and click the first corner to finish. Esc cancels the
							part. Corners snap to the corners and edges of other areas.
						{:else}
							Click a part to select it. Drag a corner to move it, drag a point between corners to
							add one, and right-click a corner to remove it.
						{/if}
					</p>
					<div class="pair">
						<Button
							kind="tertiary"
							size="md"
							fill
							disabled={edit.drawing}
							onclick={() => map?.drawPart()}>{edit.parts === 0 ? 'Draw part' : 'Add part'}</Button
						>
						<Button
							kind="danger-tertiary"
							size="md"
							fill
							disabled={!edit.partSelected}
							onclick={() => map?.removePart()}>Remove part</Button
						>
					</div>
					<div class="buttons">
						<Button type="submit" disabled={busy} fill>Save</Button>
						<Button kind="secondary" fill onclick={() => (editing = null)}>Cancel</Button>
					</div>
				</form>
			{:else if selected}
				<div class="inset panel">
					<h2 class="section">{selected.name}</h2>
					{#if clipped?.id === selected.id}
						<InlineNotification
							kind="info"
							title={`Left out where it overlapped ${clipped.by.join(', ')}.`}
						/>
					{/if}
					<div class="pair">
						<span class="wide-only">
							<Button kind="tertiary" fill onclick={() => startEditing(selected.id)}
								>Edit area</Button
							>
						</span>
						<Modal
							title="Delete area"
							triggerKind="danger-tertiary"
							submitLabel="Delete area"
							danger
							action="?/delete"
							error={form && 'deleteMessage' in form ? (form.deleteMessage as string) : undefined}
							onsuccess={() => {
								const name = shownName;
								goto(resolve('/areas'), { invalidateAll: true }).then(() =>
									toast(`Area ${name} deleted.`)
								);
							}}
						>
							<!-- The dialog is still closing once the area is gone. -->
							<input type="hidden" name="id" value={selected?.id} />
							<p>
								Every flight's time and distance per area is recomputed without {shownName}, so past
								figures change. A deleted area can't be restored.
							</p>
						</Modal>
					</div>
					<Link href={resolve('/areas')}>All areas</Link>
				</div>
			{:else if data.areas.length === 0}
				<EmptyState
					title="No areas yet"
					body="Areas are named parts of the land, such as a reserve's sections or a client's sites. Flights then show their time and distance in each."
					action={addArea}
				/>
			{:else}
				<DataTable
					label="Areas"
					{columns}
					rows={data.areas}
					rowKey={(area) => area.id}
					rowHref={(area) => areaHref(area.id)}
					summary={`${data.areas.length} ${data.areas.length === 1 ? 'area' : 'areas'}`}
				>
					{#snippet toolbar()}
						{@render addArea()}
					{/snippet}
				</DataTable>
			{/if}
		</div>
	</Column>
	<Column md={8} lg={12} narrow>
		<AreaMap
			bind:this={map}
			areas={data.areas}
			token={data.mapboxToken}
			label="Map of the areas"
			selected={selected?.id ?? null}
			{editing}
			fallback={data.fallback}
			onselect={(id) => showArea(id)}
			onedit={(state) => (edit = state)}
		/>
	</Column>
</Grid>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
	}

	/* Carbon's in-page form: fields 32px apart. */
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	/* The column has no gutter (narrow mode): containers hang into it, but
	   type and fields line up with the page's text, 16px in. */
	.inset {
		padding-inline: var(--cds-spacing-05);
	}

	.section {
		margin: 0;
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
	}

	.instructions {
		margin: 0;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* Carbon's fluid pair for Save and Cancel: equal halves, 1px apart. */
	.buttons {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
	}

	/* The other buttons stacked at full width: side by side their labels
	   wrap in this narrow column. */
	.pair {
		display: grid;
		gap: var(--cds-spacing-03);
	}

	.pair > :global(*),
	.pair :global(.button) {
		width: 100%;
		max-width: none;
	}

	/* Drawing needs a precise pointer and room: from md only. */
	.phone-only {
		display: none;
	}

	@media (max-width: 41.98rem) {
		.wide-only {
			display: none;
		}

		.phone-only {
			display: block;
		}
	}
</style>
