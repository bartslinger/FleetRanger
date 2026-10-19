<!--
	@component
	Carbon's data table. Cells show `column.format(row)` or `row[column.key]`,
	unless a `cell` snippet renders them; empty values read as "None". Sortable
	headers are links, so sorting works through the URL. Scrolls sideways when
	too wide. Carbon replaces an empty table with an empty state, so the page
	shows one instead of an empty table.
-->
<script lang="ts" module>
	export interface Column<T> {
		key: keyof T & string;
		header: string;
		/** Right-aligned with tabular figures. */
		numeric?: boolean;
		/** Serial numbers and other codes: tabular figures with a slashed zero. */
		identifier?: boolean;
		sortable?: boolean;
		format?: (row: T) => string | null | undefined;
	}

	export interface Sort {
		key: string;
		descending: boolean;
	}
</script>

<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import ArrowDown from '@carbon/icons/es/arrow--down/16.js';
	import ArrowUp from '@carbon/icons/es/arrow--up/16.js';
	import ArrowsVertical from '@carbon/icons/es/arrows--vertical/16.js';
	import Checkbox from './Checkbox.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		/** Names the table for screen readers. */
		label: string;
		columns: Column<T>[];
		rows: T[];
		/** Stable identity of a row. */
		rowKey: (row: T) => string;
		cell?: Snippet<[row: T, column: Column<T>]>;
		size?: 'sm' | 'md' | 'lg';
		/** The current sort, if the table is sortable. */
		sort?: Sort;
		/** The order when the user hasn't chosen one. */
		defaultSort?: Sort;
		/** Link for a header click; `undefined` returns to the default order. */
		sortHref?: (next: Sort | undefined) => string;
		/** Opens a row's page: the first cell becomes a link that covers the row. */
		rowHref?: (row: T) => string;
		/** Carbon's table toolbar: filters, then the primary action at the right. */
		toolbar?: Snippet;
		/** What the table shows, e.g. "37 flights", at the toolbar's left. */
		summary?: string;
		/** Rows that can be selected for batch actions; none if not given. */
		selectable?: (row: T) => boolean;
		/** Keys of the selected rows. */
		selected?: string[];
		/** Carbon's batch actions, shown over the toolbar while rows are selected. */
		batchActions?: Snippet<[selected: string[]]>;
		/** E.g. "3 flights selected"; with `all`, all that match ("All 340 flights selected"). */
		selectedLabel?: (count: number, all: boolean) => string;
		/** How many rows match across all pages. With more than this page
		 * shows, a fully selected page offers selecting all of them. */
		total?: number;
		/** All matching rows are selected, not only `selected`. */
		allMatching?: boolean;
		/** The offer, e.g. "Select all 340 flights". */
		selectAllLabel?: (total: number) => string;
	}

	let {
		label,
		columns,
		rows,
		rowKey,
		cell,
		size = 'md',
		sort,
		defaultSort,
		sortHref,
		rowHref,
		toolbar,
		summary,
		selectable,
		selected = $bindable([]),
		batchActions,
		selectedLabel = (count, all) => (all ? `All ${count} selected` : `${count} selected`),
		total,
		allMatching = $bindable(false),
		selectAllLabel = (total) => `Select all ${total}`
	}: Props = $props();

	const selectableKeys = $derived(selectable ? rows.filter(selectable).map(rowKey) : []);
	const allSelected = $derived(
		selectableKeys.length > 0 && selectableKeys.every((key) => selected.includes(key))
	);

	function toggle(key: string, on: boolean) {
		if (allMatching && !on) {
			// Out of "all matching": this page's rows, but for this one.
			allMatching = false;
			selected = selectableKeys.filter((other) => other !== key);
			return;
		}
		selected = on ? [...selected, key] : selected.filter((other) => other !== key);
	}

	function toggleAll(on: boolean) {
		allMatching = false;
		selected = on ? selectableKeys : [];
	}

	function cancel() {
		allMatching = false;
		selected = [];
	}

	// The whole page is selected and more rows match: offer them all.
	const offerAll = $derived(
		!allMatching && total !== undefined && allSelected && total > selectableKeys.length
	);

	/**
	 * Carbon's cycle: unsorted, ascending, descending, back to the default order.
	 * When that default is this column's descending order, the last step would
	 * change nothing, so it goes to ascending instead.
	 */
	function nextSort(key: string): Sort | undefined {
		if (sort?.key !== key) return { key, descending: false };
		if (!sort.descending) return { key, descending: true };
		const isDefault = defaultSort?.key === key && defaultSort.descending;
		return isDefault ? { key, descending: false } : undefined;
	}

	function ariaSort(key: string) {
		if (sort?.key !== key) return undefined;
		return sort.descending ? 'descending' : 'ascending';
	}

	function value(row: T, column: Column<T>) {
		const value = column.format ? column.format(row) : row[column.key];
		return value === null || value === undefined || value === '' ? undefined : String(value);
	}
</script>

{#if (selected.length > 0 || allMatching) && batchActions}
	<!-- Carbon's batch actions: the count, the actions, and Cancel to deselect.
	     A fully selected page offers all matching rows (not Carbon: it doesn't
	     cover selecting across pages). -->
	<div class="toolbar batch" role="region" aria-label="Batch actions">
		<p class="count" aria-live="polite">
			{allMatching && total !== undefined
				? selectedLabel(total, true)
				: selectedLabel(selected.length, false)}
		</p>
		{#if offerAll && total !== undefined}
			<button class="select-all" type="button" onclick={() => (allMatching = true)}>
				{selectAllLabel(total)}
			</button>
		{/if}
		{@render batchActions(selected)}
		<button class="cancel" type="button" onclick={cancel}>Cancel</button>
	</div>
{:else if toolbar || selectable || summary}
	<!-- Without its own items, the toolbar keeps the batch bar's place, so the
	     table doesn't move when the first row is selected. -->
	<div class="toolbar">
		{#if summary}<p class="summary" aria-live="polite">{summary}</p>{/if}
		{#if toolbar}{@render toolbar()}{/if}
	</div>
{/if}

{#snippet content(row: T, column: Column<T>)}
	{#if cell}
		{@render cell(row, column)}
	{:else}
		{@const text = value(row, column)}
		{#if text === undefined}
			<span aria-hidden="true">—</span><span class="visually-hidden">None</span>
		{:else}
			{text}
		{/if}
	{/if}
{/snippet}

<!-- A scrollable region must be reachable by keyboard (WCAG 2.1.1). -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="container" role="region" aria-label={label} tabindex="0">
	<table class={size}>
		<caption class="visually-hidden">{label}</caption>
		<thead>
			<tr>
				{#if selectable}
					<th scope="col" class="select">
						<Checkbox
							label="Select all"
							hideLabel
							checked={allMatching || allSelected}
							indeterminate={!allMatching && selected.length > 0 && !allSelected}
							disabled={selectableKeys.length === 0}
							onchange={(event) => toggleAll(event.currentTarget.checked)}
						/>
					</th>
				{/if}
				{#each columns as column (column.key)}
					<th scope="col" class:numeric={column.numeric} aria-sort={ariaSort(column.key)}>
						{#if column.sortable && sortHref}
							{@const current = sort?.key === column.key}
							<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- built by the caller -->
							<a class="sort" class:current href={sortHref(nextSort(column.key))}>
								{column.header}
								<span class="sort-icon">
									<Icon
										icon={current ? (sort?.descending ? ArrowDown : ArrowUp) : ArrowsVertical}
									/>
								</span>
							</a>
						{:else}
							<span class="header">{column.header}</span>
						{/if}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row (rowKey(row))}
				{@const key = rowKey(row)}
				{@const chosen = allMatching ? !!selectable?.(row) : selected.includes(key)}
				<tr class:linked={rowHref} class:selected={chosen}>
					{#if selectable}
						<td class="select">
							<Checkbox
								label="Select row"
								hideLabel
								checked={chosen}
								disabled={!selectable(row)}
								onchange={(event) => toggle(key, event.currentTarget.checked)}
							/>
						</td>
					{/if}
					{#each columns as column, index (column.key)}
						<td class:numeric={column.numeric} class:identifier={column.identifier}>
							{#if index === 0 && rowHref}
								<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- built by the caller -->
								<a class="row-link" href={rowHref(row)}>{@render content(row, column)}</a>
							{:else}
								{@render content(row, column)}
							{/if}
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	/* Positioned so it also clips visually hidden text in the cells, which is
	   absolutely positioned; otherwise that text widens the page on phones. */
	.container {
		position: relative;
		max-width: 100%;
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		background: var(--cds-layer-01);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	th,
	td {
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-05);
		text-align: left;
		white-space: nowrap;
	}

	.sm th,
	.sm td {
		height: var(--cds-size-small);
	}

	.lg th,
	.lg td {
		height: var(--cds-size-large);
	}

	th {
		background: var(--cds-layer-accent-01);
		color: var(--cds-text-primary);
		font-weight: var(--cds-heading-compact-01-font-weight);
	}

	/* Sortable headers fill the cell, as Carbon's sort buttons do. */
	th:has(.sort) {
		padding: 0;
	}

	.sort {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-03);
		height: 100%;
		padding: 0 var(--cds-spacing-05);
		color: inherit;
		text-decoration: none;
	}

	.numeric .sort {
		justify-content: flex-end;
	}

	.sort:hover {
		background: var(--cds-layer-accent-hover-01);
		color: var(--cds-text-primary);
	}

	.sort:active {
		background: var(--cds-layer-accent-active-01);
	}

	.sort-icon {
		display: flex;
		color: var(--cds-icon-primary);
	}

	/* Only the sorted column shows its arrow; the others on hover and focus. */
	.sort:not(.current) .sort-icon {
		visibility: hidden;
	}

	.sort:hover .sort-icon,
	.sort:focus-visible .sort-icon {
		visibility: visible;
	}

	td {
		border-top: 1px solid var(--cds-border-subtle-01);
		color: var(--cds-text-secondary);
	}

	tbody tr:hover td {
		background: var(--cds-layer-hover-01);
		color: var(--cds-text-primary);
	}

	.numeric {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	.identifier {
		font-variant-numeric: tabular-nums slashed-zero;
	}

	/* A row that opens a page: the link in its first cell covers the whole row,
	   so a click anywhere opens it and the focus outline frames the row. One
	   tab stop per row, with real link semantics. */
	tr.linked {
		position: relative;
		cursor: pointer;
	}

	.row-link {
		color: var(--cds-link-primary);
		text-decoration: none;
	}

	.row-link::after {
		content: '';
		position: absolute;
		inset: 0;
	}

	tr.linked:hover .row-link {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}

	.row-link:focus-visible {
		outline: none;
	}

	.row-link:focus-visible::after {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Carbon's table toolbar: 48px on the table's layer, its items at the right,
	   the primary button last and flush with the edge. */
	.toolbar {
		display: flex;
		justify-content: flex-end;
		align-items: center;
		min-height: var(--cds-size-large);
		background: var(--cds-layer-01);
	}

	/* Batch actions: the brand colour, the count at the left, the actions as
	   primary buttons and Cancel at the right. */
	.batch {
		background: var(--cds-background-brand);
		color: var(--cds-text-on-color);
	}

	/* What the table shows (not Carbon: it leaves a result count open), at
	   the left like the batch bar's count. */
	.summary {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* The count always shows, on one line, cut short if it must; the
	   buttons never wrap (Carbon). */
	.count {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		padding-inline: var(--cds-spacing-05);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* "Select all 340 flights": a text button after the count, underlined as
	   a link on the blue bar. */
	.select-all {
		flex: none;
		min-height: var(--cds-size-large);
		padding: 0 var(--cds-spacing-05);
		border: 0;
		background: transparent;
		color: var(--cds-text-on-color);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		text-decoration: underline;
		white-space: nowrap;
		cursor: pointer;
	}

	.select-all:hover {
		background: var(--cds-button-primary-hover);
	}

	/* On phones the offer takes a line of its own under the bar's, so the
	   count keeps its room. */
	@media (max-width: 41.99rem) {
		.batch {
			flex-wrap: wrap;
		}

		.select-all {
			order: 1;
			flex-basis: 100%;
			min-height: var(--cds-size-medium);
			text-align: left;
		}
	}

	.select-all:focus-visible {
		outline: 2px solid var(--cds-text-on-color);
		outline-offset: -2px;
	}

	.batch :global(.button),
	.cancel {
		flex: none;
		white-space: nowrap;
	}

	/* On phones the buttons' room for an icon goes to the count. */
	@media (max-width: 41.99rem) {
		.batch :global(.button) {
			padding-inline: var(--cds-spacing-05);
		}
	}

	.cancel {
		position: relative;
		min-height: var(--cds-size-large);
		padding: 0 var(--cds-spacing-05);
		border: 0;
		background: transparent;
		color: var(--cds-text-on-color);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		cursor: pointer;
	}

	/* The divider between the actions and Cancel. */
	.cancel::before {
		content: '';
		position: absolute;
		inset: var(--cds-spacing-05) auto var(--cds-spacing-05) 0;
		width: 1px;
		background: var(--cds-text-on-color);
	}

	.cancel:hover {
		background: var(--cds-button-primary-hover);
	}

	/* White on the blue bar in both themes; focus-inverse is blue in dark. */
	.cancel:focus-visible {
		outline: 2px solid var(--cds-text-on-color);
		outline-offset: -2px;
	}

	/* The selection column: a 48px cell, the checkbox 16px from the edge. */
	.select {
		width: var(--cds-size-large);
		padding-inline: var(--cds-spacing-05) 0;
	}

	/* Above the row link, which covers the row, and the whole cell is the
	   checkbox's click target: a click just beside the box selects the row
	   instead of opening its page. */
	td.select {
		position: relative;
		z-index: 1;
	}

	td.select :global(label::after) {
		content: '';
		position: absolute;
		inset: 0;
	}

	tbody tr.selected td {
		background: var(--cds-layer-selected-01);
		color: var(--cds-text-primary);
	}
</style>
