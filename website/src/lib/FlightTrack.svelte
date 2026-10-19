<!--
	@component
	A made-up flight, drawn as the app draws tracks (magenta with a casing,
	take-off as a ring, landing as a dot) but on a bare grid instead of a map.
	It draws itself once on load, unless the reader prefers reduced motion.
-->
<script lang="ts">
	// Invented: no real place or flight.
	const path =
		'M118 318 C150 262 142 196 206 158 S326 96 376 124 S462 214 518 172 S588 72 524 58 S404 70 364 178 S300 300 232 330 S146 352 132 326';
	const grid = { width: 640, height: 400, step: 80 };
	const columns = Array.from({ length: grid.width / grid.step + 1 }, (_, i) => i * grid.step);
	const rows = Array.from({ length: grid.height / grid.step + 1 }, (_, i) => i * grid.step);

	const facts = [
		{ term: 'Take-off', value: '06:12' },
		{ term: 'Landing', value: '06:41' },
		{ term: 'In the air', value: '29 min' },
		{ term: 'Distance', value: '6.4 km' }
	];
</script>

<figure>
	<svg viewBox="0 0 {grid.width} {grid.height}" aria-hidden="true">
		<g class="grid">
			{#each columns as x (x)}
				<line x1={x} y1="0" x2={x} y2={grid.height} />
			{/each}
			{#each rows as y (y)}
				<line x1="0" y1={y} x2={grid.width} y2={y} />
			{/each}
		</g>
		<path class="casing" d={path} pathLength="1" />
		<path class="track" d={path} pathLength="1" />
		<circle class="takeoff" cx="118" cy="318" r="7" />
		<circle class="landing" cx="132" cy="326" r="6" />
		<g class="scale">
			<line x1="512" y1="372" x2="608" y2="372" />
			<line x1="512" y1="366" x2="512" y2="378" />
			<line x1="608" y1="366" x2="608" y2="378" />
			<text x="560" y="354">500 m</text>
		</g>
	</svg>
	<figcaption>
		<span class="caption">Example flight</span>
		<dl>
			{#each facts as fact (fact.term)}
				<div>
					<dt>{fact.term}</dt>
					<dd>{fact.value}</dd>
				</div>
			{/each}
		</dl>
	</figcaption>
</figure>

<style>
	figure {
		position: relative;
		background: var(--cds-layer-01);
	}

	/* On the drawing's top left, like a map's label. */
	.caption {
		position: absolute;
		top: var(--cds-spacing-05);
		left: var(--cds-spacing-05);
		padding-inline: var(--cds-spacing-02);
		background: var(--cds-layer-01);
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	svg {
		display: block;
		width: 100%;
		height: auto;
	}

	.grid line {
		stroke: var(--cds-border-subtle-01);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}

	path {
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
		stroke-dasharray: 1;
		stroke-dashoffset: 1;
		/* Carbon's expressive entrance easing, slowed down for a whole flight. */
		animation: draw 2400ms cubic-bezier(0, 0, 0.3, 1) 300ms forwards;
	}

	.casing {
		stroke: var(--cds-layer-01);
		stroke-width: 9;
	}

	.track {
		stroke: var(--track);
		stroke-width: 4;
	}

	.takeoff {
		fill: var(--cds-layer-01);
		stroke: var(--track);
		stroke-width: 3;
	}

	.landing {
		fill: var(--track);
		stroke: var(--cds-layer-01);
		stroke-width: 2;
		opacity: 0;
		animation: land 240ms cubic-bezier(0, 0, 0.3, 1) 2700ms forwards;
	}

	.scale line {
		stroke: var(--cds-text-secondary);
		stroke-width: 1.5;
	}

	.scale text {
		fill: var(--cds-text-secondary);
		/* In the drawing's units: about 12px on a phone, 18px on a laptop. */
		font-size: 20px;
		font-family: var(--cds-font-family-sans);
		text-anchor: middle;
	}

	/* base.css shortens animations to nothing; drop the waits too. */
	@media (prefers-reduced-motion: reduce) {
		path,
		.landing {
			animation-delay: 0ms;
		}
	}

	@keyframes draw {
		to {
			stroke-dashoffset: 0;
		}
	}

	@keyframes land {
		to {
			opacity: 1;
		}
	}

	dl {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--cds-spacing-05);
		margin: 0;
		padding: var(--cds-spacing-05);
		border-top: 1px solid var(--cds-border-subtle-01);
	}

	@media (min-width: 42rem) {
		dl {
			grid-template-columns: repeat(4, minmax(0, 1fr));
		}
	}

	dt {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	dd {
		margin: var(--cds-spacing-02) 0 0;
		font-size: var(--cds-heading-03-font-size);
		line-height: var(--cds-heading-03-line-height);
		font-variant-numeric: tabular-nums;
	}
</style>
