<!--
	The dialog for one of an app's roles, to add one or edit it: its name for
	admins and its key for the app, checked when the key loses focus.
-->
<script lang="ts">
	import type { IconDescriptor } from '#lib/ui/Icon.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import { keyError } from './keys.js';

	interface Props {
		app: string;
		error?: string;
		/** The role being edited; none for a new one. */
		role?: { id: string; key: string; name: string };
		/** An icon trigger in the role's row. */
		triggerIcon?: IconDescriptor;
	}

	let { app, error, role, triggerIcon }: Props = $props();

	// svelte-ignore state_referenced_locally
	let key = $state(role?.key ?? '');
	let checked = $state(false);
	const keyMessage = $derived(checked ? keyError(key) : undefined);
</script>

<Modal
	title={role ? 'Edit role' : 'Add role'}
	label={role?.name ?? app}
	{triggerIcon}
	triggerSize={triggerIcon ? 'md' : undefined}
	triggerLabel={role ? `Edit role ${role.name}` : undefined}
	submitLabel={role ? 'Save' : 'Add'}
	action="?/role"
	{error}
	onsuccess={() => toast(role ? 'Role saved.' : 'Role added.')}
>
	{#if role}<input type="hidden" name="id" value={role.id} />{/if}
	<TextInput
		label="Name"
		name="name"
		value={role?.name}
		helper="What admins see when they give the role, e.g. Report viewer."
		required
		maxlength={100}
		autocomplete="off"
	/>
	<TextInput
		label="Key"
		name="key"
		bind:value={key}
		helper={role
			? `What ${app} checks for. Changing it breaks ${app}’s checks until ${app} uses the new key too.`
			: `What ${app} checks for, exactly as its instructions give it: lowercase letters, digits and underscores, starting with a letter.`}
		error={keyMessage}
		onblur={() => (checked = true)}
		required
		maxlength={64}
		spellcheck={false}
		autocomplete="off"
		autocapitalize="off"
	/>
</Modal>
