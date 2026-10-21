<script lang="ts">
	import { deserialize } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import type { SavedArea } from '#lib/server/backend.js';
	import AreaMap, { type Mode } from '#lib/ui/AreaMap.svelte';
	import Add from '@carbon/icons/es/add/16.js';
	import Button from '#lib/ui/Button.svelte';
	import type { Area, Boundary } from '#lib/ui/areas.js';
	import Column from '#lib/ui/Column.svelte';
	import ContainedList from '#lib/ui/ContainedList.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import OverflowMenu from '#lib/ui/OverflowMenu.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let map: AreaMap | undefined = $state();
	let selected: string | null = $state(null);
	/** What the map is doing; while drawing, naming or editing, the list waits. */
	let mode: Mode = $state('view');

	// The area the row menu acts on, and the dialogs it opens.
	let acting: Area | null = $state(null);
	let renaming = $state(false);
	let deleting = $state(false);

	const items = $derived(data.areas.map((area) => ({ id: area.id, label: area.name, area })));

	/** Sends a drawn shape to a form action; an error message if refused. */
	async function send(action: string, fields: Record<string, string>) {
		const body = new FormData();
		for (const [key, value] of Object.entries(fields)) body.set(key, value);
		const response = await fetch(`?/${action}`, {
			method: 'POST',
			body,
			headers: { 'x-sveltekit-action': 'true' }
		});
		const result = deserialize(await response.text());
		if (result.type === 'failure') return String(result.data?.message ?? 'Saving failed.');
		if (result.type !== 'success') return 'Saving failed. Try again.';
		const saved = result.data?.saved as SavedArea;
		await invalidateAll();
		selected = saved.areas[0].id;
		const name = saved.areas[0].name;
		const split = saved.areas.length > 1 ? ` It became ${saved.areas.length} areas.` : '';
		const clipped = saved.clipped_by.length
			? ` Left out where it overlapped ${saved.clipped_by.join(', ')}.`
			: '';
		toast(`Area ${name} saved.${clipped}${split}`);
		return undefined;
	}

	const create = (name: string, boundary: Boundary) =>
		send('create', { name, boundary: JSON.stringify(boundary) });
	const reshape = (id: string, boundary: Boundary) =>
		send('reshape', { id, boundary: JSON.stringify(boundary) });
</script>

<PageHeader title="Areas" />

<Grid rowGap>
	<Column md={8} lg={4} narrow>
		<ContainedList
			label="Areas"
			title={items.length === 0
				? 'No areas yet'
				: `${items.length} ${items.length === 1 ? 'area' : 'areas'}`}
			empty="Flights show their time and distance in each area."
			{items}
			{selected}
			disabled={mode !== 'view'}
			onselect={(id) => (selected = id)}
		>
			{#snippet headerAction()}
				<!-- In the list it adds to; the same as the map's New area tool.
				     Phones only view. -->
				<span class="wide-only">
					{#if mode === 'drawing'}
						<Button kind="ghost" size="sm" onclick={() => map?.stop()}>Cancel</Button>
					{:else}
						<Button
							kind="ghost"
							size="sm"
							icon={Add}
							disabled={mode !== 'view'}
							onclick={() => map?.newArea()}>New area</Button
						>
					{/if}
				</span>
			{/snippet}
			{#snippet action(item)}
				<OverflowMenu
					label={`Options for ${item.label}`}
					items={[
						{
							label: 'Rename',
							onselect: () => {
								acting = item.area;
								renaming = true;
							}
						},
						{
							label: 'Delete',
							danger: true,
							onselect: () => {
								acting = item.area;
								deleting = true;
							}
						}
					]}
				/>
			{/snippet}
		</ContainedList>
	</Column>
	<Column md={8} lg={12} narrow>
		<AreaMap
			bind:this={map}
			areas={data.areas}
			token={data.mapboxToken}
			label="Map of the areas"
			{selected}
			fallback={data.fallback}
			onselect={(id) => (selected = id)}
			onmode={(value) => (mode = value)}
			oncreate={create}
			onreshape={reshape}
		/>
	</Column>
</Grid>

<Modal
	title="Rename area"
	submitLabel="Save"
	action="?/rename"
	trigger={false}
	bind:open={renaming}
	error={form && 'renameMessage' in form ? (form.renameMessage as string) : undefined}
	onsuccess={() => toast('Area renamed.')}
>
	<input type="hidden" name="id" value={acting?.id} />
	<TextInput label="Name" name="name" value={acting?.name} required autocomplete="off" />
</Modal>

<Modal
	title="Delete area"
	submitLabel="Delete area"
	danger
	action="?/delete"
	trigger={false}
	bind:open={deleting}
	error={form && 'deleteMessage' in form ? (form.deleteMessage as string) : undefined}
	onsuccess={() => {
		if (selected === acting?.id) selected = null;
		toast(`Area ${acting?.name} deleted.`);
	}}
>
	<input type="hidden" name="id" value={acting?.id} />
	<p>
		Every flight's time and distance per area is recomputed without {acting?.name}, so past figures
		change. A deleted area can't be restored.
	</p>
</Modal>

<style>
	/* Drawing needs a precise pointer and room: from md only. */
	@media (max-width: 41.98rem) {
		.wide-only {
			display: none;
		}
	}
</style>
