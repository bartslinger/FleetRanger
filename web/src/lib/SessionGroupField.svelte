<!--
	@component
	A new session's group, posted as `group_id` (an id, or "organisation"):
	a select when there is a choice, preselected with the flights' group,
	otherwise the one possible group, without a field (user's decision).
-->
<script lang="ts">
	import Select from '#lib/ui/Select.svelte';
	import type { sessionGroupChoice } from '#lib/groups.js';

	interface Props {
		choice: ReturnType<typeof sessionGroupChoice>;
		/** The form it belongs to, when it sits outside it. */
		form?: string;
	}

	let { choice, form }: Props = $props();
</script>

{#if choice.options.length > 1}
	<Select
		label="Group"
		name="group_id"
		{form}
		options={choice.options}
		value={choice.value}
		placeholder={choice.value ? undefined : 'Choose a group'}
		helper="Editors of this group can change the session."
	/>
{:else if choice.options.length === 1}
	<input type="hidden" name="group_id" {form} value={choice.value} />
{/if}
