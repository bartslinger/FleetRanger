<!--
	The controller app's download page, public: opened on a DJI controller to
	install the app and hand it this install's address. Login links here.
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

	const setupLink = $derived(`fleetranger://setup?${new URLSearchParams({ server: data.server })}`);
</script>

<PageHeader
	title="Controller app"
	description="FleetRanger on a DJI controller: the pilot picks who's flying, and the flight logs upload by themselves."
/>

<Grid>
	<Column lg={8}>
		{#if data.download.android}
			<ol class="steps">
				<li>
					<h2>Download the app</h2>
					<p>Open this page on the controller and download the app there.</p>
					<div class="action">
						<!-- eslint-disable svelte/no-navigation-without-resolve -- a backend file, not a page -->
						<Button href="/api/download/FleetRanger.apk" download>Download app</Button>
						<!-- eslint-enable svelte/no-navigation-without-resolve -->
						<span class="detail">FleetRanger.apk, {fileSize(data.download.android.size_bytes)}</span
						>
					</div>
				</li>
				<li>
					<h2>Install it</h2>
					<p>
						Open the downloaded file. Android first asks to allow installing apps from the browser:
						allow it, then install.
					</p>
				</li>
				<li>
					<h2>Connect it to this FleetRanger</h2>
					<p>Open the app from here, so it knows where to connect.</p>
					<div class="action">
						<!-- eslint-disable svelte/no-navigation-without-resolve -- opens the app -->
						<Button href={setupLink} kind="tertiary">Open FleetRanger</Button>
						<!-- eslint-enable svelte/no-navigation-without-resolve -->
					</div>
					<p>Or type this address in the app: <code>{data.server}</code></p>
				</li>
				<li>
					<h2>Pair it</h2>
					<p>
						The app shows a code. Anyone with a FleetRanger account approves it at
						<a href={resolve('/pair')}>{data.server}/pair</a>.
					</p>
				</li>
			</ol>
		{:else}
			<InlineNotification kind="info" title="The app isn't available yet">
				This FleetRanger has no controller app to download. Ask your FleetRanger admin.
			</InlineNotification>
		{/if}
	</Column>
</Grid>

<style>
	/* Numbered steps, 32px apart; the number belongs to the step's heading. */
	.steps {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
		counter-reset: step;
	}

	.steps > li {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		counter-increment: step;
	}

	h2 {
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-02-font-size);
		font-weight: var(--cds-heading-compact-02-font-weight);
		line-height: var(--cds-heading-compact-02-line-height);
		letter-spacing: var(--cds-heading-compact-02-letter-spacing);
	}

	h2::before {
		content: counter(step) '. ';
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
		margin-block: var(--cds-spacing-03);
	}

	.detail {
		color: var(--cds-text-secondary);
		font-size: var(--cds-helper-text-01-font-size);
		line-height: var(--cds-helper-text-01-line-height);
		letter-spacing: var(--cds-helper-text-01-letter-spacing);
	}
</style>
