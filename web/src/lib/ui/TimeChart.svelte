<!--
	@component
	One series of readings over time, as Carbon's line chart (Carbon Charts) on
	a time axis, in a container on layer-01. Each reading is a point, as the
	line between two only guides the eye; the y axis fits the readings, as
	Carbon allows for line charts, so slow change shows. No legend for one
	series, no toolbar (carbon-expert's advice). The chart is drawn in the
	browser; the page should list the same values in a table, which is the
	chart's accessible alternative.

	Deviation: Carbon Charts sets axis labels in IBM Plex Sans Condensed, which
	the app doesn't bundle; they use the app's Plex Sans.
-->
<script lang="ts">
	import { ChartTheme, ScaleTypes } from '@carbon/charts';
	import { LineChart, type LineChartOptions } from '@carbon/charts-svelte';
	import '@carbon/charts-svelte/styles.min.css';

	interface Props {
		title: string;
		/** The axes' titles, with units, e.g. "Full charge capacity (mAh)". */
		xTitle: string;
		yTitle: string;
		readings: { at: string; value: number }[];
		/** A reading's value in the tooltip, e.g. "4,230 mAh". */
		formatValue: (value: number) => string;
		/** A reading's time in the tooltip. */
		formatTime: (at: string) => string;
		/** For screen readers: what the chart shows and where to read it instead. */
		description: string;
	}

	let { title, xTitle, yTitle, readings, formatValue, formatTime, description }: Props = $props();

	const id = $props.id();

	// The palette follows the theme as it changes: chosen in the app (the
	// root's data-theme) or, on "system", in the device's settings.
	let dark = $state(false);
	$effect(() => {
		const device = matchMedia('(prefers-color-scheme: dark)');
		const follow = () => {
			const chosen = document.documentElement.dataset.theme;
			dark = chosen ? chosen === 'dark' : device.matches;
		};
		follow();
		const observer = new MutationObserver(follow);
		observer.observe(document.documentElement, { attributeFilter: ['data-theme'] });
		device.addEventListener('change', follow);
		return () => {
			observer.disconnect();
			device.removeEventListener('change', follow);
		};
	});

	const data = $derived(
		readings.map((reading) => ({ group: yTitle, date: new Date(reading.at), value: reading.value }))
	);

	const options: LineChartOptions = $derived({
		axes: {
			bottom: { title: xTitle, mapsTo: 'date', scaleType: ScaleTypes.TIME },
			left: { title: yTitle, mapsTo: 'value', includeZero: false }
		},
		legend: { enabled: false },
		toolbar: { enabled: false },
		points: { enabled: true, radius: 3 },
		curve: 'curveLinear',
		height: '320px',
		// Carbon's palettes for light and dark containers (layer-01).
		theme: dark ? ChartTheme.G90 : ChartTheme.WHITE,
		accessibility: { svgAriaLabel: description },
		tooltip: {
			valueFormatter: (value: unknown) =>
				value instanceof Date ? formatTime(value.toISOString()) : formatValue(Number(value))
		}
	});
</script>

<figure class="chart" aria-labelledby="{id}-title">
	<figcaption id="{id}-title" class="title">{title}</figcaption>
	<LineChart {data} {options} />
</figure>

<style>
	/* A container like the data table's: layer-01, its title in heading-03
	   with 16px around and 24px below. */
	.chart {
		margin: 0;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
		--cds-charts-font-family: var(--cds-font-family-sans);
		--cds-charts-font-family-condensed: var(--cds-font-family-sans);
	}

	.title {
		margin-bottom: var(--cds-spacing-06);
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
		color: var(--cds-text-primary);
	}
</style>
