<script lang="ts">
	import { resolve } from '$app/paths';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import LoginForm from '#lib/ui/LoginForm.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Shown while the browser goes to the login provider. Reset when it comes
	// back to this page from the browser's cache, e.g. after cancelling there.
	let leaving = $state(false);
</script>

<svelte:window onpageshow={() => (leaving = false)} />

<Grid>
	<Column md={4} lg={4}>
		<LoginForm
			title="Log in to FleetRanger"
			description="Use your organisation's Google account."
			helper="Accounts are added by your organisation's admin."
		>
			{#if data.error === 'no-access'}
				<InlineNotification kind="error" title="No access">
					This Google account can't use FleetRanger. Ask your organisation's FleetRanger admin for
					access.
				</InlineNotification>
			{:else if data.error === 'failed'}
				<InlineNotification kind="error" title="Couldn't log in">
					Something went wrong. Try again.
				</InlineNotification>
			{:else if data.loggedOut}
				<InlineNotification kind="success" title="You've logged out." />
			{/if}
			<!-- eslint-disable svelte/no-navigation-without-resolve -- a backend route, not a page -->
			<Button
				href={data.loginHref}
				fill
				data-sveltekit-reload
				aria-disabled={leaving}
				onclick={() => (leaving = true)}
			>
				{leaving ? 'Logging in…' : 'Log in with Google'}
			</Button>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</LoginForm>
		<p class="controller">
			Setting up a DJI controller? <a href={resolve('/download')}>Download the app</a>
		</p>
	</Column>
</Grid>

<style>
	/* Apart from the login: it's for the controller, not for logging in. */
	.controller {
		margin-top: var(--cds-spacing-09);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}
</style>
