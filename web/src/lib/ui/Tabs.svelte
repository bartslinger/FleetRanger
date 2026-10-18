<!--
	@component
	Carbon's contained tabs, attached to a panel on the same layer, such as a
	table. Bits UI gives one tab stop, arrow keys that wrap, and selection that
	follows focus (automatic activation).
-->
<script lang="ts" generics="V extends string">
	import type { Snippet } from 'svelte';
	import { Tabs } from 'bits-ui';

	interface Tab {
		value: V;
		label: string;
	}

	interface Props {
		tabs: Tab[];
		value: V;
		onchange?: (value: V) => void;
		/** The panel of the selected tab. */
		panel: Snippet<[value: V]>;
	}

	let { tabs, value = $bindable(), onchange, panel }: Props = $props();
</script>

<Tabs.Root bind:value onValueChange={(next) => onchange?.(next as V)} activationMode="automatic">
	<Tabs.List class="tabs-list">
		{#each tabs as tab (tab.value)}
			<Tabs.Trigger class="tab" value={tab.value}>{tab.label}</Tabs.Trigger>
		{/each}
	</Tabs.List>
	{#each tabs as tab (tab.value)}
		<Tabs.Content value={tab.value}>
			{#if tab.value === value}{@render panel(tab.value)}{/if}
		</Tabs.Content>
	{/each}
</Tabs.Root>

<style>
	/* Scrolls sideways rather than wrapping, as Carbon's tabs do. */
	:global(.tabs-list) {
		display: flex;
		overflow-x: auto;
		scrollbar-width: none;
	}

	/* 40px high, 16px padding; unselected on layer-accent with a divider. */
	:global(.tab) {
		flex: none;
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-05);
		border: 0;
		border-top: 2px solid transparent;
		border-right: 1px solid var(--cds-border-strong-01);
		background: var(--cds-layer-accent-01);
		color: var(--cds-text-secondary);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		white-space: nowrap;
		cursor: pointer;
	}

	:global(.tab:last-child) {
		border-right: 0;
	}

	:global(.tab:hover) {
		background: var(--cds-layer-accent-hover-01);
		color: var(--cds-text-primary);
	}

	:global(.tab[data-state='active']) {
		border-top-color: var(--cds-border-interactive);
		border-right-color: transparent;
		background: var(--cds-layer-01);
		color: var(--cds-text-primary);
		font-weight: var(--cds-heading-compact-01-font-weight);
	}

	:global(.tab:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}
</style>
