<!--
	@component
	The page header: the page title, an optional description and page actions,
	on the grid. Unlike Carbon's page header it has no band; it sits on the
	page background (see docs/design-system.md). The inner geometry is Carbon's.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { cubicOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';
	import { trail } from '#lib/trail.svelte.js';
	import Breadcrumb, { type Crumb } from './Breadcrumb.svelte';
	import Column from './Column.svelte';
	import Grid from './Grid.svelte';
	import Tag from './Tag.svelte';

	interface Props {
		title: string;
		/** The pages above this one, for a page about one item: the breadcrumb
		 * when the page was opened directly. Otherwise it shows the pages the
		 * user came through (see trail.svelte.ts). */
		breadcrumb?: Crumb[];
		/** The page's name in a later page's breadcrumb, when the title alone
		 * wouldn't say what it is, e.g. "Flight 12 Oct, 18:53". */
		crumb?: string;
		/** A state shown as a gray tag after the title, e.g. "Archived". */
		status?: string;
		description?: string;
		/** Below md, instead of `description`: e.g. facts that on wider
		 * screens show beside the content, but on phones only further down. */
		phoneDescription?: string;
		actions?: Snippet;
	}

	let { title, breadcrumb, crumb, status, description, phoneDescription, actions }: Props =
		$props();

	$effect(() => {
		trail.title = crumb ?? title;
	});

	const crumbs: Crumb[] | undefined = $derived(
		trail.steps.length > 0
			? trail.steps.map((step) => ({
					href: step.href,
					label: step.label,
					follow: () => trail.go(step)
				}))
			: breadcrumb
	);
</script>

<svelte:head>
	<title>{title} · FleetRanger</title>
</svelte:head>

<header>
	<Grid>
		<Column>
			<div class="content">
				<div class="text">
					<!-- Appearing on a page already shown (a session opened over the
					     Sessions cards), it slides open rather than pushing the page
					     down at once: Carbon's moderate-01, about its standard curve
					     (user's choice, 2026-10-21, over keeping its row free). -->
					{#if crumbs}<div
							class="breadcrumb"
							transition:slide={{ duration: 150, easing: cubicOut }}
						>
							<Breadcrumb items={crumbs} />
						</div>
					{/if}
					<div class="title">
						<h1>{title}</h1>
						{#if status}<Tag>{status}</Tag>{/if}
					</div>
					{#if phoneDescription}
						<p class="phone">{phoneDescription}</p>
						{#if description}<p class="wide">{description}</p>{/if}
					{:else if description}
						<p>{description}</p>
					{/if}
				</div>
				{#if actions}<div class="actions">{@render actions()}</div>{/if}
			</div>
		</Column>
	</Grid>
</header>

<style>
	/* 32px from the shell header, as Carbon's content inset, and 32px to the
	   content below. */
	header {
		padding-block: var(--cds-spacing-07);
	}

	.content {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: var(--cds-spacing-05);
	}

	.text {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
		max-width: 40rem;
	}

	/* 16px to the title (with the gap), keeping it attached to the title. */
	.breadcrumb {
		margin-bottom: var(--cds-spacing-03);
	}

	.title {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--cds-spacing-03);
	}

	h1 {
		font-size: var(--cds-heading-04-font-size);
		font-weight: var(--cds-heading-04-font-weight);
		line-height: var(--cds-heading-04-line-height);
		letter-spacing: var(--cds-heading-04-letter-spacing);
		text-wrap: balance;
	}

	p {
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	@media (max-width: 41.99rem) {
		.wide {
			display: none;
		}

		/* The actions always on a line of their own below the title, not
		   squeezed beside a short one. */
		.text {
			flex-basis: 100%;
		}
	}

	@media (min-width: 42rem) {
		.phone {
			display: none;
		}
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-03);
	}
</style>
