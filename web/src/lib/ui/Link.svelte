<!--
	@component
	Carbon's standalone link, on its own after the content it belongs to: the
	link colour, underlined only on hover, focus and when pressed. Body-compact
	size, as the values around it.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	interface Props extends HTMLAnchorAttributes {
		href: string;
		children: Snippet;
	}

	let { children, ...rest }: Props = $props();
</script>

<a class="link" {...rest}>{@render children()}</a>

<style>
	.link {
		color: var(--cds-link-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
		outline: none;
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
