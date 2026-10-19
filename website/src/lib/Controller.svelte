<script lang="ts">
	// A screenshot on the screen of a photo of a DJI RC Pro.
	import rcPro400 from '#lib/images/rc-pro-400.webp';
	import rcPro765 from '#lib/images/rc-pro-765.webp';

	let { src, srcset, alt }: { src: string; srcset: string; alt: string } = $props();
</script>

<div class="controller">
	<img
		class="photo"
		src={rcPro765}
		srcset="{rcPro400} 400w, {rcPro765} 765w"
		sizes="24rem"
		alt=""
		width="765"
		height="574"
	/>
	<img class="screen" {src} {srcset} sizes="18rem" {alt} width="1920" height="1080" />
</div>

<style>
	.controller {
		position: relative;
	}

	img {
		display: block;
		height: auto;
	}

	.photo {
		width: 100%;
	}

	/* The screen in the photo: 580 × 326 px from (92, 238), of 765 px wide. Its
	   offset from the top is a margin, which like the others is relative to the
	   width. */
	.screen {
		position: absolute;
		top: 0;
		left: calc(92 / 765 * 100%);
		width: calc(580 / 765 * 100%);
		margin-top: calc(238 / 765 * 100%);
	}
</style>
