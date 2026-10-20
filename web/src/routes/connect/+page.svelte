<!--
	The consent page: an app (e.g. an AI assistant) asks to act for the user.
	It lists what the app will and won't be able to do, so admins see that the
	app gets less than they can do themselves.
-->
<script lang="ts">
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
		'flights:read':
			'Read summaries of your flights and sessions, and those of your teams: times, air time, distance, pilots and equipment'
	};

	// Admins and the all-flights role see every team's flights in FleetRanger,
	// but an app acting for them doesn't.
	const seesAllTeams = $derived(
		data.account?.role === 'admin' || data.account?.role === 'all_flights'
	);
	const limits = $derived([
		'Change anything',
		...(seesAllTeams ? ['See flights of other teams'] : []),
		'See tracks, locations or raw flight logs'
	]);

	// The answer goes with the app's request as the server checked it, never
	// the raw query, which could name another action.
	const action = (name: string) => `?${data.query}&/${name}`;
</script>

<svelte:head>
	<title>Connect an app · FleetRanger</title>
</svelte:head>

<Grid>
	<Column>
		{#if data.request}
			{@const app = data.request.app}
			<DecisionTile
				title="Connect {app} to FleetRanger?"
				description="{app} ({data.request.host}) asks to act for you, {data.account
					?.name}. What it reads goes to {app}."
				helper="You can disconnect it at any time under Connected apps in your profile."
			>
				<PermissionList
					heading="{app} will be able to"
					items={data.request.scopes.map((scope) => ABILITIES[scope] ?? scope)}
					granted
				/>
				<PermissionList heading="{app} will not be able to" items={limits} granted={false} />
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
				<InlineNotification kind="error" title={data.problem ?? 'The request is invalid.'}>
					Go back to the app and try again. If it keeps happening, the app may not work with
					FleetRanger.
				</InlineNotification>
			</DecisionTile>
		{/if}
	</Column>
</Grid>
