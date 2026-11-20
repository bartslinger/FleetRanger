<!--
	The documentation's pages: their list beside the text from lg (columns 1-4,
	the text in 5-12), and above it below lg, folded into a <details>, which
	works without scripts. The list follows Carbon's UI shell left panel: two
	tiers, the sections as headings over their pages, the current page
	selected.
-->
<script lang="ts">
	import { page } from '$app/state';
	import ChevronDown from '@carbon/icons/es/chevron--down/16.js';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import { sections } from '#lib/docs.js';

	let { children } = $props();

	const current = $derived(
		sections.flatMap((s) => s.pages).find((p) => page.url.pathname === href(p.section, p.slug))
	);

	function href(section: string, slug: string): string {
		return `/docs/${section}/${slug}`;
	}
</script>

{#snippet list()}
	{#each sections as section (section.slug)}
		<p class="heading">{section.title}</p>
		<ul>
			{#each section.pages as p (p.slug)}
				<li>
					<a href={href(section.slug, p.slug)} aria-current={p === current ? 'page' : undefined}
						>{p.title}</a
					>
				</li>
			{/each}
		</ul>
	{/each}
{/snippet}

<div class="docs">
	<Grid centered>
		<Column md={8} lg={4}>
			<nav class="wide" aria-label="Documentation">
				{@render list()}
			</nav>
			<details class="narrow">
				<summary>{current?.title ?? 'Documentation'}<Icon icon={ChevronDown} /></summary>
				<nav aria-label="Documentation">
					{@render list()}
				</nav>
			</details>
		</Column>
		<Column md={6} lg={8}>
			{@render children()}
		</Column>
	</Grid>
</div>

<style>
	.docs {
		padding-block: var(--cds-spacing-07) var(--cds-spacing-12);
	}

	.wide {
		display: none;
	}

	@media (min-width: 66rem) {
		.wide {
			display: block;
			position: sticky;
			top: var(--cds-spacing-07);
		}

		.narrow {
			display: none;
		}
	}

	.heading {
		padding: var(--cds-spacing-05) var(--cds-spacing-05) var(--cds-spacing-03);
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.heading:first-child {
		padding-top: 0;
	}

	ul {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	/* The left panel's links: 32px, secondary text; the current page selected,
	   with Carbon's marker at its start. */
	a {
		display: flex;
		align-items: center;
		min-height: var(--cds-spacing-08);
		padding-inline: var(--cds-spacing-05);
		border-inline-start: 3px solid transparent;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
	}

	a:hover {
		background: var(--cds-background-hover);
		color: var(--cds-text-primary);
	}

	a[aria-current='page'] {
		border-inline-start-color: var(--cds-border-interactive);
		background: var(--cds-background-selected);
		color: var(--cds-text-primary);
		font-weight: 600;
	}

	a:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Below lg: Carbon's accordion header, 48px, with its chevron. */
	.narrow {
		margin-bottom: var(--cds-spacing-07);
		border-block: 1px solid var(--cds-border-subtle-01);
	}

	summary {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: var(--cds-size-large);
		padding-inline: var(--cds-spacing-05);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
		list-style: none;
	}

	summary::-webkit-details-marker {
		display: none;
	}

	/* The chevron turns up while the list is open. */
	details[open] summary :global(svg) {
		transform: rotate(180deg);
	}

	summary:hover {
		background: var(--cds-layer-hover-01);
	}

	summary:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	details nav {
		padding-bottom: var(--cds-spacing-05);
	}
</style>
