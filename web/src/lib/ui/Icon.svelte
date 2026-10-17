<!--
	@component
	Renders an icon from @carbon/icons, e.g.
	`import ErrorFilled from '@carbon/icons/es/error--filled/16.js'`.
	Decorative unless `label` is given.
-->
<script lang="ts" module>
	export interface IconDescriptor {
		attrs: { viewBox: string; width: number; height: number };
		content: { elem: string; attrs: Record<string, string | number> }[];
	}
</script>

<script lang="ts">
	interface Props {
		icon: IconDescriptor;
		label?: string;
	}

	let { icon, label }: Props = $props();
</script>

<svg
	viewBox={icon.attrs.viewBox}
	width={icon.attrs.width}
	height={icon.attrs.height}
	fill="currentColor"
	role={label ? 'img' : undefined}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
	focusable="false"
>
	{#each icon.content as child, i (i)}
		<svelte:element this={child.elem} {...child.attrs} />
	{/each}
</svg>

<style>
	svg {
		flex: none;
	}
</style>
