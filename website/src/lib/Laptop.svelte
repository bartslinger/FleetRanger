<script lang="ts">
	// A screenshot on the screen of a picture of a laptop.
	import laptop800 from '#lib/images/laptop-800.webp';
	import laptop1600 from '#lib/images/laptop-1600.webp';

	// width: how wide it is shown at most, in rem, for picking the image sizes.
	let {
		src,
		srcset,
		alt,
		width = 32
	}: { src: string; srcset: string; alt: string; width?: number } = $props();
</script>

<div class="laptop">
	<img
		class="picture"
		src={laptop1600}
		srcset="{laptop800} 800w, {laptop1600} 1600w"
		sizes="{width}rem"
		alt=""
		width="3571"
		height="2087"
	/>
	<img class="screen" {src} {srcset} sizes="{Math.ceil(width * 0.76)}rem" {alt} />
</div>

<style>
	.laptop {
		position: relative;
	}

	img {
		display: block;
	}

	.picture {
		width: 100%;
		height: auto;
	}

	/* The screen in the picture: 2704 × 1660 px from (437, 120), of 3571 × 2087,
	   over the picture's menu bar. A wider screenshot loses its right edge. */
	.screen {
		position: absolute;
		top: calc(120 / 2087 * 100%);
		left: calc(437 / 3571 * 100%);
		width: calc(2704 / 3571 * 100%);
		height: calc(1660 / 2087 * 100%);
		object-fit: cover;
		object-position: left top;
	}
</style>
