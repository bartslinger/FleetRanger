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
	import { DropdownMenu } from 'bits-ui';
	import OverflowMenuHorizontal from '@carbon/icons/es/overflow-menu--horizontal/16.js';
	import { MediaQuery } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { plainClick } from '#lib/click.js';
	import Icon from './Icon.svelte';

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
				<DropdownMenu.Root>
					<DropdownMenu.Trigger class="overflow-trigger" aria-label="Earlier pages">
						<Icon icon={OverflowMenuHorizontal} />
					</DropdownMenu.Trigger>
					<DropdownMenu.Portal>
						<DropdownMenu.Content
							class="overflow-menu"
							align="start"
							sideOffset={4}
							collisionPadding={16}
						>
							{#each parts[1] as item, index (index)}
								<DropdownMenu.Item class="overflow-item" onSelect={() => select(item)}>
									{item.label}
								</DropdownMenu.Item>
							{/each}
						</DropdownMenu.Content>
					</DropdownMenu.Portal>
				</DropdownMenu.Root>
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

	/* The overflow menu's trigger: an icon-only ghost button, as high as the
	   breadcrumb's text. */
	:global(.overflow-trigger) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--cds-spacing-06);
		height: var(--cds-spacing-05);
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	/* Larger on phones, for touch. */
	@media (max-width: 41.99rem) {
		:global(.overflow-trigger) {
			width: var(--cds-spacing-07);
			height: var(--cds-spacing-06);
		}
	}

	:global(.overflow-trigger:hover),
	:global(.overflow-trigger[data-state='open']) {
		background: var(--cds-background-hover);
	}

	:global(.overflow-trigger:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Carbon's overflow menu, medium: 40px options on the layer, with a
	   shadow; rendered in a portal. */
	:global(.overflow-menu) {
		z-index: 10;
		min-width: 10rem;
		max-width: 20rem;
		background: var(--cds-layer-01);
		box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.3);
	}

	:global(.overflow-item) {
		display: flex;
		align-items: center;
		min-height: 2.5rem;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
		outline: none;
	}

	:global(.overflow-item[data-highlighted]) {
		background: var(--cds-layer-hover-01);
		color: var(--cds-text-primary);
	}

	:global(.overflow-item:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}
</style>
