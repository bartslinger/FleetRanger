<!--
	@component
	Carbon's standalone link, on its own after the content it belongs to: the
	link colour, underlined only on hover, focus and when pressed. Body-compact
	size, as the values around it, or `size="lg"` beside 16px body copy. An
	`icon` (16px, 20px at lg) follows the text 8px after it, in its colour.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import Icon, { type IconDescriptor } from './Icon.svelte';

	interface Props extends HTMLAnchorAttributes {
		href: string;
		size?: 'md' | 'lg';
		icon?: IconDescriptor;
		children: Snippet;
	}

	let { size = 'md', icon, children, ...rest }: Props = $props();
</script>

<a class="link {size}" {...rest}
	>{@render children()}{#if icon}<span class="icon"><Icon {icon} /></span>{/if}</a
>

<style>
	.link {
		color: var(--cds-link-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
		outline: none;
	}

	.link.lg {
		font-size: var(--cds-body-compact-02-font-size);
		line-height: var(--cds-body-compact-02-line-height);
		letter-spacing: var(--cds-body-compact-02-letter-spacing);
	}

	/* Carbon's icon: one line high, on the text's bottom, 8px after it. */
	.icon {
		display: inline-flex;
		align-items: center;
		height: 1lh;
		margin-inline-start: var(--cds-spacing-03);
		vertical-align: text-bottom;
	}

	.lg .icon :global(svg) {
		width: 1.25rem;
		height: 1.25rem;
	}

	.link:hover {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}

	.link:focus-visible {
		text-decoration: underline;
		outline: 1px solid var(--cds-focus);
	}

	.link:active {
		color: var(--cds-text-primary);
		text-decoration: underline;
	}
</style>
