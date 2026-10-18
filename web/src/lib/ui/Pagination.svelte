<!--
	@component
	Carbon's pagination bar, placed directly below a data table at the same
	size. Navigates through links built by `href`, so the page and size live in
	the URL. Below the md breakpoint only the range and previous/next remain.
-->
<script lang="ts">
	import CaretLeft from '@carbon/icons/es/caret--left/16.js';
	import CaretRight from '@carbon/icons/es/caret--right/16.js';
	import ChevronDown from '@carbon/icons/es/chevron--down/16.js';
	import { goto } from '$app/navigation';
	import Icon from './Icon.svelte';

	interface Props {
		/** 1-based. */
		page: number;
		size: number;
		total: number;
		sizes: number[];
		/** Link to a page at a page size. */
		href: (page: number, size: number) => string;
		/** What is being paged, plural ("items" by default). */
		noun?: string;
	}

	let { page, size, total, sizes, href, noun = 'items' }: Props = $props();

	const pages = $derived(Math.max(1, Math.ceil(total / size)));
	const first = $derived(total === 0 ? 0 : (page - 1) * size + 1);
	const last = $derived(Math.min(page * size, total));

	/** Thousands separators without Intl, so server and browser agree. */
	const count = (value: number) => String(value).replace(/\B(?=(\d{3})+(?!\d))/g, ',');

	function go(path: string) {
		// Keep the scroll position and focus on the select.
		// eslint-disable-next-line svelte/no-navigation-without-resolve -- built by the caller
		goto(path, { reset: false });
	}
</script>

<nav class="pagination" aria-label="Pagination">
	<div class="left">
		<label class="size">
			<span class="label">Items per page:</span>
			<span class="select">
				<select value={size} onchange={(event) => go(href(1, Number(event.currentTarget.value)))}>
					{#each sizes as option (option)}
						<option value={option}>{option}</option>
					{/each}
				</select>
				<Icon icon={ChevronDown} />
			</span>
		</label>
		<span class="range">{count(first)}–{count(last)} of {count(total)} {noun}</span>
	</div>
	<div class="right">
		<label class="page">
			<span class="visually-hidden">Page</span>
			<span class="select">
				<select
					value={page}
					onchange={(event) => go(href(Number(event.currentTarget.value), size))}
				>
					{#each { length: pages }, i (i)}
						<option value={i + 1}>{i + 1}</option>
					{/each}
				</select>
				<Icon icon={ChevronDown} />
			</span>
			<span class="label">of {count(pages)} {pages === 1 ? 'page' : 'pages'}</span>
		</label>
		<!-- eslint-disable svelte/no-navigation-without-resolve -- links are built by the caller -->
		{#if page > 1}
			<a
				class="step"
				href={href(page - 1, size)}
				aria-label="Previous page"
				data-sveltekit-noscroll
			>
				<Icon icon={CaretLeft} />
			</a>
		{:else}
			<span class="step disabled" aria-label="Previous page" aria-disabled="true" role="link">
				<Icon icon={CaretLeft} />
			</span>
		{/if}
		{#if page < pages}
			<a class="step" href={href(page + 1, size)} aria-label="Next page" data-sveltekit-noscroll>
				<Icon icon={CaretRight} />
			</a>
		{:else}
			<span class="step disabled" aria-label="Next page" aria-disabled="true" role="link">
				<Icon icon={CaretRight} />
			</span>
		{/if}
		<!-- eslint-enable svelte/no-navigation-without-resolve -->
	</div>
</nav>

<style>
	.pagination {
		display: flex;
		justify-content: space-between;
		min-height: var(--cds-size-medium);
		background: var(--cds-layer-01);
		border-top: 1px solid var(--cds-border-subtle-01);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* Sections fill the bar's height, so their dividers do too. */
	.left,
	.right {
		display: flex;
		align-items: stretch;
	}

	.range,
	label {
		display: flex;
		align-items: center;
	}

	.size {
		display: none;
		height: 100%;
		padding-left: var(--cds-spacing-05);
		border-right: 1px solid var(--cds-border-subtle-01);
	}

	.range {
		padding: 0 var(--cds-spacing-05);
		font-variant-numeric: tabular-nums;
	}

	.page {
		display: none;
		height: 100%;
		padding-right: var(--cds-spacing-05);
		border-left: 1px solid var(--cds-border-subtle-01);
	}

	.page .label {
		font-variant-numeric: tabular-nums;
	}

	/* A borderless inline select, as in Carbon's pagination. */
	.select {
		position: relative;
		display: flex;
		align-items: center;
		height: 100%;
		color: var(--cds-text-primary);
	}

	.select :global(svg) {
		position: absolute;
		right: var(--cds-spacing-03);
		pointer-events: none;
	}

	select {
		height: 100%;
		padding: 0 var(--cds-spacing-07) 0 var(--cds-spacing-05);
		border: 0;
		background: transparent;
		color: var(--cds-text-primary);
		font-variant-numeric: tabular-nums;
		appearance: none;
		cursor: pointer;
	}

	select:hover {
		background: var(--cds-layer-hover-01);
	}

	option {
		background: var(--cds-layer-01);
	}

	.step {
		display: grid;
		place-items: center;
		width: var(--cds-size-medium);
		border-left: 1px solid var(--cds-border-subtle-01);
		color: var(--cds-icon-primary);
	}

	a.step:hover {
		background: var(--cds-layer-hover-01);
	}

	a.step:active {
		background: var(--cds-layer-active-01);
	}

	.disabled {
		color: var(--cds-icon-disabled);
		cursor: not-allowed;
	}

	/* From Carbon's md breakpoint: items per page and the page selector. */
	@media (min-width: 42rem) {
		.size,
		.page {
			display: flex;
		}
	}
</style>
