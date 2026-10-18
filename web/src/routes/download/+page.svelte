<!--
	The controller app's download page, public: opened on a DJI controller to
	install the app, with the address to type in it. Login links here.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { fileSize } from '#lib/format.js';
	import Button from '#lib/ui/Button.svelte';
	import Column from '#lib/ui/Column.svelte';
	import Grid from '#lib/ui/Grid.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<PageHeader
	title="Controller app"
	description="FleetRanger on a DJI controller: the pilot picks who's flying, and the flight logs upload by themselves."
/>

<Grid>
	<Column lg={8}>
		{#if data.download.android}
			<div class="content">
				<div class="action">
					<!-- eslint-disable svelte/no-navigation-without-resolve -- a backend file, not a page -->
					<Button href="/api/download/FleetRanger.apk" download>Download app</Button>
					<!-- eslint-enable svelte/no-navigation-without-resolve -->
					<span class="detail">FleetRanger.apk, {fileSize(data.download.android.size_bytes)}</span>
				</div>
				<p>
					On the controller, install the app and open it. It connects to this FleetRanger by itself;
					if it asks for the address, enter:
				</p>
				<p class="address"><code>{data.address}</code></p>
				<p>
					The app then shows a code. Anyone with a FleetRanger account approves it at
					<a href={resolve('/pair')}>{data.server}/pair</a>.
				</p>
			</div>
		{:else}
			<InlineNotification kind="info" title="The app isn't available yet">
				This FleetRanger has no controller app to download. Ask your FleetRanger admin.
			</InlineNotification>
		{/if}
	</Column>
</Grid>

<style>
	.content {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
	}

	p {
		max-width: 40rem;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	.action {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--cds-spacing-05);
		margin-bottom: var(--cds-spacing-05);
	}

	/* What gets typed on the controller: large enough to read from the page beside it. */
	.address {
		font-size: var(--cds-heading-03-font-size);
		line-height: var(--cds-heading-03-line-height);
	}

	.detail {
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
