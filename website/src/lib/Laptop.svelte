<script lang="ts">
	// A screenshot of the web app on the screen of a picture of a laptop, in the
	// website's theme: `light` and `dark` are the same page in each.
	import laptop800 from '#lib/images/laptop-800.webp';
	import laptop1600 from '#lib/images/laptop-1600.webp';

	interface Screenshot {
		src: string;
		srcset: string;
	}

	// width: how wide it is shown at most, in rem, for picking the image sizes.
	let {
		light,
		dark,
		alt,
		width = 32
	}: { light: Screenshot; dark: Screenshot; alt: string; width?: number } = $props();

	let sizes = $derived(`${Math.ceil(width * 0.76)}rem`);
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
	<img class="screen light" src={light.src} srcset={light.srcset} {sizes} {alt} />
	<img class="screen dark" src={dark.src} srcset={dark.srcset} {sizes} {alt} />
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
	   over the picture's menu bar. A wider screenshot sits at the top, and the
	   app's background colour fills the screen below it. */
	.screen {
		position: absolute;
		top: calc(120 / 2087 * 100%);
		left: calc(437 / 3571 * 100%);
		width: calc(2704 / 3571 * 100%);
		height: calc(1660 / 2087 * 100%);
		object-fit: contain;
		object-position: top;
	}

	/* The app's background colours: Carbon's Gray 10 and Gray 100, with the side
	   navigation (96 of the screenshot's 3600 px) continuing in Gray 100. */
	.light {
		background: linear-gradient(to right, #161616 calc(96 / 3600 * 100%), #f4f4f4 0);
	}

	.dark {
		display: none;
		background: #161616;
	}

	@media (prefers-color-scheme: dark) {
		:global(:root:not([data-theme='light'])) .light {
			display: none;
		}

		:global(:root:not([data-theme='light'])) .dark {
			display: block;
		}
	}

	:global(:root[data-theme='dark']) .light {
		display: none;
	}

	:global(:root[data-theme='dark']) .dark {
		display: block;
	}
</style>
