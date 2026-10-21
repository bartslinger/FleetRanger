<!--
	@component
	A session's day from 00:00 to 24:00 as a thin strip: a mark per flight
	from takeoff to landing (at least 2px), night (sunset to sunrise at the
	first takeoff point) as a faint tint, and notches at 06:00, 12:00 and
	18:00 outside the outline. A flight past midnight wraps around to the
	left (user's decision, 2026-10-21), so every strip has the same scale.

	Our own chart, as Carbon has none for this; one colour for all flights,
	which are one kind of thing, and no legend or labels (carbon-expert's
	review). Hovering a flight shows its times in a tooltip. The strip sits
	in a clickable tile, so nothing in it can be focusable: the tooltip is
	for the mouse only, and the strip is hidden from screen readers. That's
	acceptable because it's a shortcut: the session's page lists every
	flight's times.
-->
<script lang="ts" module>
	/** A flight on the strip; `from` is its takeoff point, [lon, lat, alt]. */
	export interface DayStripFlight {
		takeoff_at: string;
		landed_at: string;
		air_time_s: number;
		from: number[] | null;
	}
</script>

<script lang="ts">
	import { Tooltip } from 'bits-ui';
	import { clock, duration } from '#lib/format.js';
	import { sunriseSunset } from '#lib/sun.js';
	import TooltipContent from './TooltipContent.svelte';

	interface Props {
		flights: DayStripFlight[];
		timeZone: string;
	}

	let { flights, timeZone }: Props = $props();

	const DAY = 24 * 60;

	/** The minutes since midnight of an instant, in the time zone. */
	function minutes(iso: string): number {
		const [hour, minute] = clock(iso, timeZone).split(':').map(Number);
		return hour * 60 + minute;
	}

	const percent = (minutes: number) => `${(minutes / DAY) * 100}%`;

	// Night as [start, end] in minutes, where the first flight took off.
	const nights = $derived.by(() => {
		const first = flights[0];
		const times = first?.from && sunriseSunset(first.takeoff_at, first.from[0], first.from[1]);
		if (!times) return [];
		const [rise, set] = times.map((time) => minutes(time.toISOString()));
		return [
			[0, rise],
			[set, DAY]
		];
	});

	// In minutes, with the tooltip's text; a flight past midnight becomes two.
	const marks = $derived(
		flights.flatMap(({ takeoff_at, landed_at, air_time_s }) => {
			const start = minutes(takeoff_at);
			const end = minutes(landed_at);
			const label = `${clock(takeoff_at, timeZone)}–${clock(landed_at, timeZone)} · ${duration(air_time_s)}`;
			return end >= start
				? [{ start, end, label }]
				: [
						{ start, end: DAY, label },
						{ start: 0, end, label }
					];
		})
	);

	// Each mark's hover area: spacing-02 wider on each side, but never past
	// the middle of the gap to its neighbour, so the closest mark wins.
	const hits = $derived.by(() => {
		const sorted = marks.toSorted((a, b) => a.start - b.start);
		return sorted.map(({ start, end, label }, index) => {
			const before = sorted[index - 1];
			const after = sorted[index + 1];
			const left = `calc(${percent(start)} - var(--cds-spacing-02))`;
			const right = `calc(100% - ${percent(end)} - var(--cds-spacing-02))`;
			return {
				start,
				end,
				label,
				left: before ? `max(${left}, ${percent((before.end + start) / 2)})` : left,
				right: after ? `max(${right}, calc(100% - ${percent((end + after.start) / 2)}))` : right
			};
		});
	});

	// The marks, for their tooltips to point at.
	const anchors: HTMLElement[] = $state([]);
</script>

<span class="strip" aria-hidden="true">
	{#each nights as [start, end], index (index)}
		<span class="night" style:left={percent(start)} style:width={percent(end - start)}></span>
	{/each}
	{#each [6, 12, 18] as hour (hour)}
		<span class="tick" style:left={percent(hour * 60)}></span>
	{/each}
	<!-- 100ms, as the icon buttons: moving along the strip, the times follow quickly. -->
	<Tooltip.Provider delayDuration={100}>
		{#each hits as { start, end, label, left, right }, index (index)}
			<Tooltip.Root>
				<!-- A span, not a button: nothing focusable inside the tile's link. -->
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<span {...props} class="hit" style:left style:right></span>
					{/snippet}
				</Tooltip.Trigger>
				<span
					class="mark"
					style:left={percent(start)}
					style:width={percent(end - start)}
					bind:this={anchors[index]}
				></span>
				<!-- Pointing at the mark, not the hover area, which a neighbour can
				     cut short on one side. -->
				<TooltipContent side="top" anchor={anchors[index]} arrow>{label}</TooltipContent>
			</Tooltip.Root>
		{/each}
	</Tooltip.Provider>
</span>

<style>
	/* An outline, no fill: a hovered or marked tile has the same grey as a
	   filled strip would. */
	.strip {
		position: relative;
		display: block;
		height: var(--cds-spacing-03);
		border: 1px solid var(--cds-border-strong-01);
	}

	/* A tint over whatever the tile's background is, so night stays a step
	   apart on a hovered or marked tile too. Carbon has no token for it. */
	.night {
		position: absolute;
		top: 0;
		bottom: 0;
		background: color-mix(in srgb, var(--cds-text-primary) 10%, transparent);
	}

	/* Only outside the outline (1px): spacing-01 above and below, never
	   across a flight. */
	.tick {
		position: absolute;
		top: calc(-1px - var(--cds-spacing-01));
		bottom: calc(-1px - var(--cds-spacing-01));
		width: 1px;
	}

	.tick::before,
	.tick::after {
		content: '';
		position: absolute;
		left: 0;
		width: 1px;
		height: var(--cds-spacing-01);
		background: var(--cds-border-strong-01);
	}

	.tick::before {
		top: 0;
	}

	.tick::after {
		bottom: 0;
	}

	/* Neutral, so the marks don't outshout the text beside them. */
	.mark {
		position: absolute;
		top: 0;
		bottom: 0;
		min-width: var(--cds-spacing-01);
		background: var(--cds-icon-secondary);
		pointer-events: none;
	}

	/* The tooltip's trigger: taller than the mark, and as wide as `hits` allows. */
	.hit {
		position: absolute;
		top: calc(-1 * var(--cds-spacing-03));
		bottom: calc(-1 * var(--cds-spacing-03));
	}

	.hit:hover + .mark {
		background: var(--cds-text-primary);
	}
</style>
