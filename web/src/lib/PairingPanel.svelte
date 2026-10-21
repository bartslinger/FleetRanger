<!--
	@component
	The controllers waiting to be paired, for the header panel, with the way to
	the page that pairs them.
-->
<script lang="ts">
	import { resolve } from '$app/paths';
	import type { WaitingController } from '#lib/server/backend.js';
	import Link from '#lib/ui/Link.svelte';
	import { startTrail } from '#lib/trail.svelte.js';

	interface Props {
		controllers: WaitingController[];
	}

	let { controllers }: Props = $props();
</script>

<section>
	<h2>Waiting to be paired</h2>
	<ul>
		{#each controllers as controller (controller.serial)}
			<li>
				<p>{controller.model}</p>
				<p class="identifier">{controller.serial}</p>
			</li>
		{/each}
	</ul>
</section>

<!-- A header panel is global, as the side navigation: no breadcrumb from here. -->
<Link href={resolve('/pair')} onclick={startTrail}>Pair a controller</Link>

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

	p {
		margin: 0;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.identifier {
		color: var(--cds-text-secondary);
		font-variant-numeric: tabular-nums slashed-zero;
	}
</style>
