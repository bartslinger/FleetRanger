<!--
	@component
	Carbon's small loading indicator (16px): a ring with a turning arc. Decorative;
	whatever is busy names it. With reduced motion the arc stands still.
-->
<svg class="loading" viewBox="0 0 100 100" width="16" height="16" aria-hidden="true">
	<circle class="track" cx="50%" cy="50%" r="42" />
	<circle class="arc" cx="50%" cy="50%" r="42" />
</svg>

<style>
	.loading {
		animation: rotate 690ms linear infinite;
	}

	circle {
		fill: none;
		stroke-width: 16;
	}

	.track {
		stroke: var(--cds-layer-accent-01);
	}

	/* A quarter of the circumference (2π × 42 ≈ 264) is drawn. */
	.arc {
		stroke: var(--cds-interactive);
		stroke-dasharray: 264;
		stroke-dashoffset: 198;
	}

	@keyframes rotate {
		to {
			transform: rotate(360deg);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.loading {
			animation: none;
		}
	}
</style>
