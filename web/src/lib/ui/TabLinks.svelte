<!--
	@component
	Carbon's line tabs, as links: each tab is a page of its own, so the chosen
	tab survives a reload, a bookmark and Back. Deviation from Carbon's tabs:
	links in a nav, each a tab stop, the current one marked with
	`aria-current="page"`, rather than a tablist with arrow keys and panels on
	the same page (carbon-expert's advice, 2026-11-13). Like the side
	navigation, a tab starts the breadcrumb trail afresh: tabs are siblings,
	not steps.
-->
<script lang="ts">
	import { page } from '$app/state';
	import { startsTrail } from '#lib/trail.svelte.js';

	interface Tab {
		/** Already passed through `resolve()`. */
		href: string;
		label: string;
	}

	let { label, tabs }: { label: string; tabs: Tab[] } = $props();

	// resolve() can return relative paths (`./`), so compare absolute ones.
	const absolute = (path: string) => new URL(path, page.url.href).pathname.replace(/\/$/, '');
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- links are resolved by the caller -->
<nav aria-label={label}>
	<ul>
		{#each tabs as tab (tab.href)}
			<li>
				<a
					use:startsTrail
					href={tab.href}
					aria-current={absolute(tab.href) === absolute(page.url.pathname) ? 'page' : undefined}
					>{tab.label}</a
				>
			</li>
		{/each}
	</ul>
</nav>

<style>
	/* Scrolls sideways rather than wrapping, as Carbon's tabs do. */
	ul {
		display: flex;
		margin: 0;
		padding: 0;
		overflow-x: auto;
		list-style: none;
		scrollbar-width: none;
	}

	/* 40px high, 16px padding, a 2px underline per tab: subtle, strong on
	   hover, interactive when selected; 70ms is fast-01. */
	a {
		display: flex;
		align-items: center;
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-05);
		border-bottom: 2px solid var(--cds-border-subtle-00);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
		white-space: nowrap;
		transition:
			border-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9),
			color 70ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	a:hover {
		border-bottom-color: var(--cds-border-strong-01);
		color: var(--cds-text-primary);
	}

	a[aria-current='page'] {
		border-bottom-color: var(--cds-border-interactive);
		color: var(--cds-text-primary);
		font-weight: var(--cds-heading-compact-01-font-weight);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	a:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}
</style>
