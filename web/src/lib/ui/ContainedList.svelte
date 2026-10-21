<!--
	@component
	Carbon's contained list with interactive rows, for a list beside a canvas
	such as a map: one item can be selected, and each row can have an action
	(e.g. an overflow menu) at its end.
-->
<script lang="ts" generics="T extends { id: string; label: string }">
	import type { Snippet } from 'svelte';

	interface Props {
		/** For screen readers, e.g. "Areas". */
		label: string;
		/** The list header's title, e.g. "3 areas"; no header without one. */
		title?: string;
		/** An action in the header, e.g. a small button to add an item. */
		headerAction?: Snippet;
		/** Shown in place of the rows when there are none. */
		empty?: string;
		items: T[];
		selected?: string | null;
		onselect?: (id: string) => void;
		/** Rows can't be chosen now, e.g. while something else is being edited. */
		disabled?: boolean;
		action?: Snippet<[item: T]>;
	}

	let {
		label,
		title,
		headerAction,
		empty,
		items,
		selected = null,
		onselect,
		disabled = false,
		action
	}: Props = $props();
</script>

{#if title}
	<!-- Carbon's on-page list header: 32px, its title and an optional action. -->
	<div class="header">
		<h2 class="title">{title}</h2>
		{#if headerAction}{@render headerAction()}{/if}
	</div>
{/if}
{#if items.length === 0 && empty}
	<p class="empty">{empty}</p>
{/if}
<ul class="list" aria-label={label}>
	{#each items as item (item.id)}
		<li class:selected={item.id === selected}>
			<button
				type="button"
				class="item"
				aria-current={item.id === selected ? 'true' : undefined}
				{disabled}
				onclick={() => onselect?.(item.id)}>{item.label}</button
			>
			{#if action}<span class="action">{@render action(item)}</span>{/if}
		</li>
	{/each}
</ul>

<style>
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: var(--cds-spacing-07);
		padding-left: var(--cds-spacing-05);
	}

	.title {
		margin: 0;
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.empty {
		margin: 0;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.list {
		margin: 0;
		padding: 0;
		list-style: none;
		background: var(--cds-layer-01);
	}

	li {
		display: flex;
		align-items: center;
		border-bottom: 1px solid var(--cds-border-subtle-01);
	}

	li:last-child {
		border-bottom: 0;
	}

	li:hover {
		background: var(--cds-layer-hover-01);
	}

	/* Selected: Carbon's selected layer, with a bar in the focus colour so it
	   isn't told by colour alone. */
	li.selected {
		background: var(--cds-layer-selected-01);
		box-shadow: inset 3px 0 0 var(--cds-border-interactive);
	}

	.item {
		flex: 1;
		min-width: 0;
		min-height: var(--cds-size-large);
		padding: 0 var(--cds-spacing-05);
		border: 0;
		background: transparent;
		color: var(--cds-text-primary);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-align: left;
		cursor: pointer;
	}

	.item:disabled {
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	.item:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* A row's action fills the row's end, e.g. a 48px icon button. */
	.action {
		display: flex;
	}
</style>
