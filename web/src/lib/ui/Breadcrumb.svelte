<!--
	@component
	Carbon's small breadcrumb, for page headers. It lists the pages before the
	current one, each followed by a slash; the current page is the title below.
	It never wraps: a long trail shows the first page and the last two, the
	ones between in an overflow menu; on phones the menu and the last page.
-->
<script lang="ts" module>
	export interface Crumb {
		/** Already passed through `resolve()`. */
		href: string;
		label: string;
		/** Goes there instead of opening the link, e.g. back through the
		 * history; a click with a modifier key still opens the link. */
		follow?: () => void;
	}
</script>

<script lang="ts">
	import { MediaQuery } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { plainClick } from '#lib/click.js';
	import OverflowMenu from './OverflowMenu.svelte';

	let { items }: { items: Crumb[] } = $props();

	const wide = new MediaQuery('min-width: 42rem');

	// Shown: [before the menu, in the menu, after it]. A menu of one page
	// would hide nothing, so short trails show in full.
	const parts = $derived.by((): [Crumb[], Crumb[], Crumb[]] => {
		if (wide.current && items.length > 4) {
			return [items.slice(0, 1), items.slice(1, -2), items.slice(-2)];
		}
		if (!wide.current && items.length > 2) return [[], items.slice(0, -1), items.slice(-1)];
		return [items, [], []];
	});

	function onclick(event: MouseEvent, item: Crumb) {
		if (!item.follow || !plainClick(event)) return;
		event.preventDefault();
		item.follow();
	}

	function select(item: Crumb) {
		if (item.follow) item.follow();
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller
		else void goto(item.href);
	}
</script>

{#snippet link(item: Crumb)}
	<li>
		<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller -->
		<a href={item.href} onclick={(event) => onclick(event, item)}>{item.label}</a>
	</li>
{/snippet}

<nav aria-label="Breadcrumb">
	<ol>
		<!-- A path can pass the same page twice. -->
		{#each parts[0] as item, index (index)}{@render link(item)}{/each}
		{#if parts[1].length > 0}
			<li>
				<OverflowMenu
					label="Earlier pages"
					items={parts[1].map((item) => ({ label: item.label, onselect: () => select(item) }))}
				/>
			</li>
		{/if}
		{#each parts[2] as item, index (index)}{@render link(item)}{/each}
	</ol>
</nav>

<style>
	ol {
		display: flex;
		flex-wrap: nowrap;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	li {
		display: flex;
		white-space: nowrap;
	}

	li::after {
		content: '/';
		margin-inline: var(--cds-spacing-02);
		color: var(--cds-text-primary);
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
		outline: 2px solid var(--cds-focus);
		outline-offset: 0;
		text-decoration: underline;
	}

	a:active {
		color: var(--cds-text-primary);
		text-decoration: underline;
	}
</style>
