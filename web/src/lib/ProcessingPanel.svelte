<!--
	@component
	What log processing is doing, for the header panel admins see: progress
	through the batch, the log being processed now (or why it's paused) and the
	latest results.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import { dateTime } from '#lib/format.js';
	import type { ProcessingStatus } from '#lib/server/backend.js';
	import ProgressBar from '#lib/ui/ProgressBar.svelte';

	interface Props {
		status: ProcessingStatus;
		timeZone: string;
	}

	let { status, timeZone }: Props = $props();

	const STAGES = {
		reading: 'Reading the file',
		requesting_keys: 'Requesting keys from DJI',
		decoding: 'Decoding',
		saving: 'Saving'
	};

	const total = $derived(status.processed + status.pending);
	const helper = $derived(
		`${status.processed} of ${total} logs` + (status.failed ? `, ${status.failed} failed` : '')
	);

	/** The file name, without the folders. */
	const fileName = (path: string) => path.split('/').pop() ?? path;
</script>

<ProgressBar label="Processing flight logs" value={status.processed} max={total} {helper} />

<section>
	<h2>Now</h2>
	{#if status.paused}
		<p>Paused until {dateTime(status.paused.until, timeZone)}</p>
		<p class="secondary">{status.paused.reason}</p>
	{:else if status.current}
		<p class="file">{fileName(status.current.path)}</p>
		<p class="secondary">{STAGES[status.current.stage]}</p>
	{:else}
		<p class="secondary">Starting the next log</p>
	{/if}
</section>

{#if status.latest.length > 0}
	<section>
		<h2>Latest</h2>
		<ul>
			{#each status.latest as log (log.path)}
				<li>
					<p class="file">{fileName(log.path)}</p>
					{#if log.status === 'failed'}
						<p class="failed">Failed: {log.error}</p>
					{:else if log.flight_id}
						<a href={resolve('/flights/[id]', { id: log.flight_id })}>View flight</a>
					{:else}
						<p class="secondary">No takeoff, so no flight</p>
					{/if}
				</li>
			{/each}
		</ul>
	</section>
{/if}

<style>
	section {
		display: grid;
		gap: var(--cds-spacing-03);
	}

	h2 {
		margin: 0;
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		font-weight: 400;
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	ul {
		display: grid;
		gap: var(--cds-spacing-04);
		margin: 0;
		padding: 0;
		list-style: none;
	}

	p,
	a {
		margin: 0;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.file {
		color: var(--cds-text-primary);
		overflow-wrap: anywhere;
	}

	.secondary {
		color: var(--cds-text-secondary);
	}

	.failed {
		color: var(--cds-text-error);
	}

	a {
		color: var(--cds-link-primary);
		text-decoration: none;
	}

	a:hover {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}

	a:focus-visible {
		outline: 1px solid var(--cds-focus);
	}
</style>
