<!--
	The apps' roles in a roles dialog, below FleetRanger's: a checkbox group per
	app, "<app> roles" as the legend beside "FleetRanger roles" (carbon-expert's
	advice; user's choice), without helper text, as only the app knows what its
	roles allow. Posted as `app_role`. Apps without roles show nothing.
-->
<script lang="ts">
	import CheckboxGroup from '#lib/ui/CheckboxGroup.svelte';
	import type { OfferedAppRole } from '#lib/server/backend.js';
	import { byApp } from './roles.js';

	interface Props {
		appRoles: OfferedAppRole[];
		/** The role ids checked at first. */
		value: string[];
	}

	let { appRoles, value }: Props = $props();
</script>

{#each byApp(appRoles) as app (app.id)}
	<CheckboxGroup
		legend="{app.name} roles"
		name="app_role"
		options={app.roles.map((role) => ({ value: role.id, label: role.name }))}
		{value}
	/>
{/each}
