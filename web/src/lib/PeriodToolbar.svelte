<!--
	@component
	The cards' toolbar, attached to them as a table's (Carbon's 48px toolbar,
	carbon-expert's check): the period between previous and next, with its
	count; at the end the list's tools (Filters) and a menu for the unit and
	the current period, which change less often (no room for a switcher in
	this column). Above the cards, where the pointer works (user's choice,
	2026-10-21). With an item open, the same bar in the same place: the
	arrows step through the items instead, in time like the periods
	(‹ earlier), on into the next period, disabled at the ends; ✕ in place of
	the list's tools goes back to the list (user's choice, 2026-10-21).
	Sessions and Flights use it.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { goto } from '$app/navigation';
	import { Tooltip } from 'bits-ui';
	import Calendar from '@carbon/icons/es/calendar/16.js';
	import Close from '@carbon/icons/es/close/16.js';
	import ChevronLeft from '@carbon/icons/es/chevron--left/16.js';
	import ChevronRight from '@carbon/icons/es/chevron--right/16.js';
	import IconButton from '#lib/ui/IconButton.svelte';
	import OverflowMenu from '#lib/ui/OverflowMenu.svelte';
	import TooltipContent from '#lib/ui/TooltipContent.svelte';
	import { roving } from '#lib/ui/roving.js';
	import { periodName, periodOf, switchTo, today, type Period } from '#lib/period.js';

	interface Props {
		/** For screen readers, e.g. "Sessions". */
		label: string;
		period: Period;
		timeZone: string;
		/** The list's URL for another period, with its filters. */
		at: (period: Period) => string;
		/** The nearest periods with items; null at the ends. */
		previousHref: string | null;
		nextHref: string | null;
		/** The name and, under it, the count (or with an item open, its day and place). */
		lines: [string, string];
		/** An item open: the arrows step through the items, ✕ goes back. */
		open?: {
			/** E.g. "session". */
			noun: string;
			earlierHref: string | null;
			laterHref: string | null;
			onclose: () => void;
			/** ✕'s name; "Back to" the period by default. */
			closeLabel?: string;
		} | null;
		/** The list's tools before the period menu, e.g. Filters. */
		tools?: Snippet;
	}

	let {
		label,
		period,
		timeZone,
		at,
		previousHref,
		nextHref,
		lines,
		open = null,
		tools
	}: Props = $props();

	const unit = $derived(period.unit);
	const other = $derived(unit === 'month' ? 'week' : 'month');
	const thisPeriod = $derived(periodOf(unit, today(timeZone)));

	// The two lines, and which are cut off: a tooltip has those in full, only
	// those (user's feedback). Checked when the width or the text changes.
	let heading: HTMLElement | undefined = $state();
	let cutLines: boolean[] = $state([false, false]);
	const cut = $derived(cutLines.some(Boolean));
	const fullText = $derived(lines.filter((_, line) => cutLines[line]).join(' · '));
	$effect(() => {
		const element = heading;
		if (!element) return;
		void lines;
		const check = () =>
			(cutLines = [...element.children].map((line) => line.scrollWidth > line.clientWidth));
		const frame = requestAnimationFrame(check);
		const resizes = new ResizeObserver(check);
		resizes.observe(element);
		return () => {
			cancelAnimationFrame(frame);
			resizes.disconnect();
		};
	});
</script>

<!-- A toolbar for screen readers, one tab stop with the arrow keys between
     its controls (Carbon's text toolbar, WAI-ARIA). -->
<div class="list-toolbar" role="toolbar" aria-label={label} use:roving>
	{#if open}
		<!-- Stepping replaces the history entry, so Back and ✕ return to the
		     list; the page keeps its scroll. -->
		{#if open.earlierHref}
			<IconButton
				icon={ChevronLeft}
				size="lg"
				label="Earlier {open.noun}"
				href={open.earlierHref}
				data-sveltekit-replacestate
				data-sveltekit-noscroll
			/>
		{:else}
			<IconButton icon={ChevronLeft} size="lg" label="Earlier {open.noun}" disabled />
		{/if}
		{#if open.laterHref}
			<IconButton
				icon={ChevronRight}
				size="lg"
				label="Later {open.noun}"
				href={open.laterHref}
				data-sveltekit-replacestate
				data-sveltekit-noscroll
			/>
		{:else}
			<IconButton icon={ChevronRight} size="lg" label="Later {open.noun}" disabled />
		{/if}
	{:else}
		<!-- The nearest periods with items: empty ones are skipped, and the
		     arrows stop at the ends (user's request, 2026-11-18). -->
		{#if previousHref}
			<IconButton icon={ChevronLeft} size="lg" label="Previous {unit}" href={previousHref} />
		{:else}
			<IconButton icon={ChevronLeft} size="lg" label="Previous {unit}" disabled />
		{/if}
		{#if nextHref}
			<IconButton icon={ChevronRight} size="lg" label="Next {unit}" href={nextHref} />
		{:else}
			<IconButton icon={ChevronRight} size="lg" label="Next {unit}" disabled />
		{/if}
	{/if}
	<!-- After both arrows, as in a calendar's header, so they stay put
	     whatever the name's length (user's feedback). Cut off, the full name
	     in a tooltip, on hover and keyboard focus (our rule for truncated
	     values; user's request). -->
	<Tooltip.Root disabled={!cut}>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<!-- One of the toolbar's stops only while cut off, so the keyboard
				     reaches the full name too; screen readers read the text. -->
				<h2
					{...props}
					bind:this={heading}
					class="period"
					aria-live="polite"
					data-roving={cut ? 'true' : 'false'}
				>
					<span class="period-name">{lines[0]}</span>
					<span class="period-count">{lines[1]}</span>
				</h2>
			{/snippet}
		</Tooltip.Trigger>
		<TooltipContent side="top">{fullText}</TooltipContent>
	</Tooltip.Root>
	<span class="tools">
		{#if open}
			<IconButton
				icon={Close}
				size="lg"
				label={open.closeLabel ?? `Back to ${periodName(period)}`}
				onclick={open.onclose}
			/>
		{:else}
			{@render tools?.()}
			<OverflowMenu
				label="Choose period"
				variant="row"
				icon={Calendar}
				items={[
					{
						label: `Show by ${other}`,
						// eslint-disable-next-line svelte/no-navigation-without-resolve -- at() resolves
						onselect: () => goto(at(switchTo(other, period, today(timeZone))))
					},
					...(period.start === thisPeriod.start
						? []
						: [
								{
									label: `Go to this ${unit}`,
									// eslint-disable-next-line svelte/no-navigation-without-resolve -- at() resolves
									onselect: () => goto(at(thisPeriod))
								}
							])
				]}
			/>
		{/if}
	</span>
</div>

<style>
	/* Carbon's large table toolbar: 48px on the list's layer, 2px above the
	   first card, its buttons full height. */
	.list-toolbar {
		display: flex;
		align-items: center;
		min-height: var(--cds-spacing-09);
		/* 2px to the first card. */
		margin-bottom: var(--cds-spacing-01);
		background: var(--cds-layer-01);
	}

	/* The buttons keep their 48px; the name gives way, its end (the count)
	   cut off with an ellipsis (user's feedback: the arrows were squeezed). */
	.list-toolbar :global(.icon-button) {
		flex: none;
	}

	/* The name, and under it the count, each line the full width so the
	   count isn't cut off behind the name (user's choice, 2026-10-21):
	   18px + 2px + 16px in the 48px bar. */
	.period {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 0.125rem;
		min-width: 0;
		margin: 0;
		padding-inline: var(--cds-spacing-03);
	}

	.period-name,
	.period-count {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.period-name {
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.period-count {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		font-weight: var(--cds-label-01-font-weight);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	.tools {
		display: flex;
		flex: none;
		margin-inline-start: auto;
	}
</style>
