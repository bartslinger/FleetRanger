<!--
	@component
	Cards of a week or month beside a map with all their tracks, under the
	period's toolbar; an item opens in the cards' place, its facts sliding in
	while the map stays and flies to it. Sessions and Flights use it. The
	cards' own markup comes from the page; the classes `month`, `card-title`,
	`card-strip`, `card-who` and `card-numbers` and the tiles' `ul` are styled
	here, as is `panel` in the facts.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** An item is open: its facts take the cards' place. */
		opened: boolean;
		/** The map: from lg, and below it only for an open item. */
		showMap: boolean;
		toolbar: Snippet;
		cards: Snippet;
		/** The open item's facts. */
		facts?: Snippet;
		map: Snippet;
	}

	let { opened, showMap, toolbar, cards, facts, map }: Props = $props();
</script>

<div class="cards-view" class:opened>
	<div class="side">
		{@render toolbar()}
		<div class="cards" inert={opened}>
			{@render cards()}
		</div>
		{#if opened && facts}
			<div class="facts-side">
				{@render facts()}
			</div>
		{/if}
	</div>
	{#if showMap}
		<div class="map-side">
			{@render map()}
		</div>
	{/if}
</div>

<style>
	/* Each month under its heading, which stays in view while its cards
	   scroll; tiles 2px apart, as in Carbon's tile groups. */
	/* minmax(0, 1fr), here and in the view: as wide as the column, never as
	   the longest content (the toolbar's name), which scrolled the page
	   sideways (user's feedback). */
	.cards {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--cds-spacing-06);
	}

	.cards :global(.month) {
		position: sticky;
		top: 3rem;
		z-index: 1;
		margin: 0;
		padding: var(--cds-spacing-03) var(--cds-spacing-05);
		background: var(--cds-background);
		color: var(--cds-text-secondary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.cards :global(section > ul) {
		display: grid;
		gap: var(--cds-spacing-01);
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.cards :global(.card-title) {
		display: block;
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.cards :global(:is(.card-who, .card-numbers)) {
		display: block;
		margin-top: var(--cds-spacing-02);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* The last line: room at its end for the tile's arrow (16px + 8px). */
	.cards :global(.card-numbers) {
		padding-inline-end: 1.5rem;
		color: var(--cds-text-secondary);
		font-variant-numeric: tabular-nums;
	}

	/* Under the times it pictures, the same width on every card, so the
	   strips line up and share one scale (user's choice, 2026-10-21: a
	   full-width strip was more than it needs; beside the title, titles
	   wrapped). */
	.cards :global(.card-strip) {
		display: block;
		width: 12rem;
		max-width: 100%;
		margin-block: var(--cds-spacing-03);
	}

	/* Only the cards below lg; from lg, 5 of 16 columns of cards and the map
	   beside them, in view while the cards scroll. */
	.cards-view {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--cds-spacing-07);
	}

	/* From lg the map on the left, 10 of 16 columns (11 from xlg), and the
	   cards on the right, 6 (5): a card's last line fits beside its arrow.
	   An item open, its facts take the cards' place and the map stays as
	   it is, only flying to the item, so the pointer stays over text
	   (user's choice, 2026-10-21). 16px between, as padding. */
	@media (min-width: 66rem) {
		.cards-view {
			/* minmax(0, …): a column never grows past its share for its content
			   (the map's canvas, a long name), which scrolled the page sideways. */
			grid-template-columns: minmax(0, 10fr) minmax(0, 6fr);
			gap: 0;
			align-items: start;
		}

		.side {
			position: relative;
			min-width: 0;
			padding-inline-start: var(--cds-spacing-05);
		}

		/* Level with the first card, 16px below the header while the cards
		   scroll, and as high as the window allows (carbon-expert's review). */
		.map-side {
			position: sticky;
			top: calc(3rem + var(--cds-spacing-05));
			order: -1;
		}

		/* Carbon's short slide: the cards leave 32px to the left with the
		   exit curve, out of the flow at once so the facts take their place;
		   the facts come from 32px to the right with the entrance curve. */
		.cards {
			transition:
				opacity 240ms cubic-bezier(0.2, 0, 1, 0.9),
				transform 240ms cubic-bezier(0.2, 0, 1, 0.9),
				visibility 0s;
		}

		.opened .cards {
			position: absolute;
			/* Under the toolbar, which stays: 48px and its 2px. */
			inset: calc(var(--cds-spacing-09) + var(--cds-spacing-01)) 0 auto var(--cds-spacing-05);
			visibility: hidden;
			opacity: 0;
			transform: translateX(-2rem);
			transition:
				opacity 240ms cubic-bezier(0.2, 0, 1, 0.9),
				transform 240ms cubic-bezier(0.2, 0, 1, 0.9),
				visibility 0s 240ms;
		}

		/* On the toolbar's layer, 2px under it, as the cards are: toolbar and
		   content one block in both states (user's feedback: the toolbar
		   floated above facts on the page background). */
		.facts-side {
			display: grid;
			gap: var(--cds-spacing-01);
			animation: 240ms cubic-bezier(0, 0, 0.38, 0.9) 120ms both facts-in;
		}
	}

	/* The facts as a panel on the toolbar's layer, 2px under it (the
	   toolbar's margin), at every width. */
	.facts-side :global(.panel) {
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
	}

	/* 16px all round: the facts' last group keeps no space of its own. */
	.facts-side :global(.panel dl:last-child) {
		margin-bottom: 0;
	}

	@keyframes facts-in {
		from {
			opacity: 0;
			transform: translateX(2rem);
		}
	}

	/* Below lg an item open shows in the cards' place: toolbar, facts,
	   map, flights, one under the other. */
	@media (max-width: 65.99rem) {
		.opened .cards {
			display: none;
		}
	}

	@media (min-width: 82rem) {
		.cards-view {
			grid-template-columns: minmax(0, 11fr) minmax(0, 5fr);
		}
	}
</style>
