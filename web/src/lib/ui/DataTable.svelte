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
		/** Link for a header click; `undefined` returns to the default order. */
		sortHref?: (next: Sort | undefined) => string;
	}

	let { label, columns, rows, rowKey, cell, size = 'md', sort, sortHref }: Props = $props();

	/** Carbon's cycle: unsorted, ascending, descending, unsorted. */
	function nextSort(key: string): Sort | undefined {
		if (sort?.key !== key) return { key, descending: false };
		return sort.descending ? undefined : { key, descending: true };
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

<!-- A scrollable region must be reachable by keyboard (WCAG 2.1.1). -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="container" role="region" aria-label={label} tabindex="0">
	<table class={size}>
		<caption class="visually-hidden">{label}</caption>
		<thead>
			<tr>
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
				<tr>
					{#each columns as column (column.key)}
						<td class:numeric={column.numeric} class:identifier={column.identifier}>
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
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.container {
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
</style>
