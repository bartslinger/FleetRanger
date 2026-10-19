<script lang="ts">
	// A screenshot on a DJI RC Pro, drawn after its proportions (268 × 137 mm,
	// a 7" screen): the folded antennas on top, the sticks beside the screen.
	let { src, srcset, alt }: { src: string; srcset: string; alt: string } = $props();
</script>

<div class="controller">
	<svg viewBox="0 0 268 150" aria-hidden="true">
		<rect class="antenna" x="24" y="2" width="86" height="14" rx="6" />
		<rect class="antenna" x="158" y="2" width="86" height="14" rx="6" />
		<rect class="body" x="0.5" y="10.5" width="267" height="139" rx="24" />
		<rect class="bezel" x="52" y="21" width="164" height="98" rx="4" />
		{#each [26, 242] as x (x)}
			<circle class="recess" cx={x} cy="114" r="13" />
			<circle class="stick" cx={x} cy="114" r="6.5" />
			<circle class="button" cx={x} cy="40" r="4" />
			<circle class="button" cx={x} cy="56" r="4" />
		{/each}
		<rect class="button" x="114" y="128" width="40" height="7" rx="3.5" />
	</svg>
	<img {src} {srcset} sizes="(min-width: 66rem) 30rem, 80vw" {alt} width="1920" height="1080" />
</div>

<style>
	.controller {
		position: relative;
	}

	svg {
		display: block;
		width: 100%;
		height: auto;
	}

	/* The controller's own dark grey, the same in both themes, with a lighter
	   edge so it stands out on the dark theme's background too. */
	.body {
		fill: #2e3033;
		stroke: #5a5d61;
	}

	.antenna {
		fill: #3a3c40;
	}

	.bezel {
		fill: #0a0a0a;
	}

	.recess {
		fill: #1d1e20;
	}

	.stick {
		fill: #45484c;
	}

	.button {
		fill: #45484c;
	}

	/* The screen inside the bezel: 155 × 87.2 mm, 16:9. Its offset from the top
	   is a margin, which like the other values is relative to the width. */
	img {
		position: absolute;
		top: 0;
		left: calc(56.5 / 268 * 100%);
		margin-top: calc(26.4 / 268 * 100%);
		width: calc(155 / 268 * 100%);
		height: auto;
	}
</style>
