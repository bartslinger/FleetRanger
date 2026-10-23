<!--
	The consent page: an app (e.g. an AI assistant) asks to act for the user.
	It lists what the app will and won't be able to do, so admins see that the
	app gets less than they can do themselves.
-->
<script lang="ts">
	import { may } from '#lib/permissions.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import DecisionTile from '#lib/ui/DecisionTile.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PermissionList from '#lib/ui/PermissionList.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	/** What each scope lets an app do. */
	const ABILITIES: Record<string, string> = {
		'flights:read': 'Read your flights and sessions, and those of your teams'
	};

	// Admins and the all-flights role see every team's flights in FleetRanger,
	// but an app acting for them doesn't.
	const seesAllTeams = $derived(may(data.account, 'flights.read', null));
	const limits = $derived([
		'Change anything',
		...(seesAllTeams ? ['See other teams’ flights'] : []),
		'See tracks or locations'
	]);

	// The answer goes with the app's request as the server checked it, never
	// the raw query, which could name another action.
	const action = (name: string) => `?${data.query}&/${name}`;
</script>

<svelte:head>
	<title>Connect an app · FleetRanger</title>
</svelte:head>

<!-- Centred on six of the grid's columns (all on phones). -->
<Grid>
	<Column md={6} mdOffset={1} lg={6} lgOffset={5}>
		{#if data.request}
			{@const app = data.request.app}
			<!-- The name is only what the app says; its address, where it sends
			     you back, is who it is. -->
			<DecisionTile
				title="Connect {app} to FleetRanger?"
				description="{data.request.host} wants to read your flight data, as {data.account?.name}."
				helper="What {app} reads is sent to {app}. You can disconnect it under Connected apps."
			>
				<PermissionList
					heading="{app} will be able to"
					items={data.request.scopes.map((scope) => ABILITIES[scope] ?? scope)}
					granted
				/>
				<PermissionList heading="{app} won’t be able to" items={limits} granted={false} />
				{#if form?.problem}
					<InlineNotification kind="error" title={form.problem} />
				{/if}
				{#snippet actions()}
					<form method="POST" action={action('allow')}>
						<Button type="submit">Allow</Button>
					</form>
					<form method="POST" action={action('cancel')}>
						<Button type="submit" kind="secondary">Cancel</Button>
					</form>
				{/snippet}
			</DecisionTile>
		{:else}
			<DecisionTile title="This app can’t connect">
				<InlineNotification kind="error" title={data.problem ?? 'Invalid request'}>
					Go back to the app and try again. If it keeps happening, the app may not work with
					FleetRanger.
				</InlineNotification>
			</DecisionTile>
		{/if}
	</Column>
</Grid>
