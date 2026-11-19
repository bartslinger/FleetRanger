<!--
	The app's error page, inside its shell: Carbon's error-management empty
	state (empty-states pattern): what happened in plain words, no codes, and
	always a way on (user's request, 2026-11-19: not a code in the corner).
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import EmptyState from '#lib/ui/EmptyState.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import Link from '#lib/ui/Link.svelte';

	const status = $derived(page.status);
	const shown = $derived(
		status === 404
			? {
					title: 'Page not found',
					body: 'This page doesn’t exist, or it isn’t one you can see.'
				}
			: status === 403
				? {
						title: 'No access',
						body: 'You don’t have permission to see this. Ask an admin of your organisation for access.'
					}
				: status < 500
					? {
							title: 'Something went wrong',
							body: 'FleetRanger couldn’t do what was asked on this page.'
						}
					: {
							title: 'Something went wrong',
							body: 'FleetRanger couldn’t load this page. Try again in a moment.'
						}
	);
	// Trying again helps only when the server had a problem.
	const retry = $derived(status >= 500);
</script>

<svelte:head><title>{shown.title} · FleetRanger</title></svelte:head>

<div class="error">
	<Grid>
		<Column narrow>
			<EmptyState level={1} title={shown.title} body={shown.body}>
				{#snippet action()}
					{#if retry}
						<Button onclick={() => location.reload()}>Try again</Button>
					{:else}
						<Link href={resolve('/')}>Go to the start page</Link>
					{/if}
				{/snippet}
			</EmptyState>
		</Column>
	</Grid>
</div>

<style>
	/* Below the shell's header with the space a page header leaves above its title. */
	.error {
		padding-top: var(--cds-spacing-07);
	}
</style>
