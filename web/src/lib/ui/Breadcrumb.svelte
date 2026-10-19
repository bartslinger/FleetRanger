<!--
	@component
	Carbon's small breadcrumb, for page headers. It lists the pages before the
	current one, each followed by a slash; the current page is the title below.
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
	import { plainClick } from '#lib/click.js';

	let { items }: { items: Crumb[] } = $props();

	function onclick(event: MouseEvent, item: Crumb) {
		if (!item.follow || !plainClick(event)) return;
		event.preventDefault();
		item.follow();
	}
</script>

<nav aria-label="Breadcrumb">
	<ol>
		<!-- A path can pass the same page twice. -->
		{#each items as item, index (index)}
			<li>
				<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller -->
				<a href={item.href} onclick={(event) => onclick(event, item)}>{item.label}</a>
			</li>
		{/each}
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
