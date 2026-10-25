<!--
	The dialog for an app's name and return addresses, to add one or edit it.
	The addresses are a text area, one per line (carbon-expert's advice: Carbon
	has no repeating field, and most apps have one), checked when it loses
	focus.
-->
<script lang="ts">
	import Modal from '#lib/ui/Modal.svelte';
	import TextArea from '#lib/ui/TextArea.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { addressError } from './addresses.js';

	interface Props {
		title: string;
		submitLabel: string;
		action: string;
		error?: string;
		/** The app being edited; none for a new one. */
		app?: { name: string; redirect_uris: string[] };
		triggerKind?: 'primary' | 'tertiary';
	}

	let { title, submitLabel, action, error, app, triggerKind }: Props = $props();

	// svelte-ignore state_referenced_locally
	let addresses = $state(app?.redirect_uris.join('\n') ?? '');
	let checked = $state(false);
	const addressMessage = $derived(checked ? addressError(addresses) : undefined);
</script>

<Modal
	{title}
	{triggerKind}
	label={app?.name}
	{submitLabel}
	{action}
	{error}
	onsuccess={() => toast(app ? 'App saved.' : 'App added. Copy its client ID into the app.')}
>
	<TextInput
		label="Name"
		name="name"
		value={app?.name}
		required
		maxlength={100}
		autocomplete="off"
	/>
	<TextArea
		label="Return addresses"
		name="redirect_uris"
		helper="Where the app receives people after they log in. Enter one web address per line, up to 10, starting with https://."
		rows={3}
		bind:value={addresses}
		error={addressMessage}
		onblur={() => (checked = true)}
		required
		spellcheck={false}
		autocomplete="off"
	/>
</Modal>
