<!--
	@component
	Carbon's data table, display only. Cells show `row[column.key]` unless a
	`cell` snippet renders them. Scrolls sideways when too wide.
-->
<script lang="ts" module>
	export interface Column<T> {
		key: keyof T & string;
		header: string;
		/** Right-aligned with tabular figures. */
		numeric?: boolean;
	}
</script>

<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';

	interface Props {
		columns: Column<T>[];
		rows: T[];
		/** Stable identity of a row. */
		rowKey: (row: T) => string;
		cell?: Snippet<[row: T, column: Column<T>]>;
		/** Shown when there are no rows. */
		empty?: string;
		size?: 'sm' | 'md' | 'lg';
	}

	let { columns, rows, rowKey, cell, empty = 'No data', size = 'lg' }: Props = $props();
</script>

<div class="container">
	<table class={size}>
		<thead>
			<tr>
				{#each columns as column (column.key)}
					<th scope="col" class:numeric={column.numeric}>{column.header}</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row (rowKey(row))}
				<tr>
					{#each columns as column (column.key)}
						<td class:numeric={column.numeric}>
							{#if cell}{@render cell(row, column)}{:else}{row[column.key]}{/if}
						</td>
					{/each}
				</tr>
			{:else}
				<tr>
					<td class="empty" colspan={columns.length}>{empty}</td>
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
		height: var(--cds-size-large);
		padding: 0 var(--cds-spacing-05);
		text-align: left;
		white-space: nowrap;
	}

	.md th,
	.md td {
		height: var(--cds-size-medium);
	}

	.sm th,
	.sm td {
		height: var(--cds-size-small);
	}

	th {
		background: var(--cds-layer-accent-01);
		color: var(--cds-text-primary);
		font-weight: var(--cds-heading-compact-01-font-weight);
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

	.empty {
		color: var(--cds-text-helper);
	}

	tbody tr:has(.empty):hover td {
		background: none;
	}
</style>
