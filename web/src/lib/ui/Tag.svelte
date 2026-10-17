<!--
	@component
	Carbon's read-only tag, for labelling and categorising.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		type?:
			| 'gray'
			| 'cool-gray'
			| 'warm-gray'
			| 'red'
			| 'magenta'
			| 'purple'
			| 'blue'
			| 'cyan'
			| 'teal'
			| 'green';
		size?: 'sm' | 'md';
		children: Snippet;
	}

	let { type = 'gray', size = 'md', children }: Props = $props();
</script>

<span
	class="tag {size}"
	style:--background="var(--cds-tag-background-{type})"
	style:--color="var(--cds-tag-color-{type})"
>
	{@render children()}
</span>

<style>
	.tag {
		display: inline-flex;
		align-items: center;
		max-width: 100%;
		min-height: var(--cds-size-xsmall);
		padding: 0 var(--cds-spacing-03);
		border-radius: var(--cds-spacing-05);
		background: var(--background);
		color: var(--color);
		font-size: var(--cds-label-01-font-size);
		font-weight: var(--cds-label-01-font-weight);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		vertical-align: middle;
	}

	.sm {
		min-height: 1.125rem;
		padding-inline: var(--cds-spacing-02);
	}
</style>
