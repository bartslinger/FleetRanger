<!--
	@component
	Carbon's button. Renders a link when `href` is given.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';

	type Props = {
		kind?: 'primary' | 'secondary' | 'tertiary' | 'ghost' | 'danger' | 'danger-tertiary';
		size?: 'sm' | 'md' | 'lg' | 'xl';
		/** Fills its container, e.g. in a form or a panel. */
		fill?: boolean;
		children: Snippet;
	} & (({ href: string } & HTMLAnchorAttributes) | ({ href?: never } & HTMLButtonAttributes));

	let { kind = 'primary', size = 'lg', fill = false, children, ...rest }: Props = $props();
</script>

{#if rest.href !== undefined}
	<a class="button {kind} {size}" class:fill {...rest as HTMLAnchorAttributes}
		>{@render children()}</a
	>
{:else}
	<button class="button {kind} {size}" class:fill type="button" {...rest as HTMLButtonAttributes}>
		{@render children()}
	</button>
{/if}

<style>
	.button {
		display: inline-flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--cds-spacing-05);
		max-width: 20rem;
		min-height: var(--cds-size-large);
		margin: 0;
		/* Carbon leaves room on the right for an icon, also when there is none. */
		padding: 0 calc(var(--cds-spacing-10) - 1px) 0 calc(var(--cds-spacing-05) - 1px);
		border: 1px solid transparent;
		border-radius: 0;
		font-size: var(--cds-body-compact-01-font-size);
		font-weight: var(--cds-body-compact-01-font-weight);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-align: left;
		text-decoration: none;
		cursor: pointer;
		transition:
			background 70ms cubic-bezier(0, 0, 0.38, 0.9),
			border-color 70ms cubic-bezier(0, 0, 0.38, 0.9),
			color 70ms cubic-bezier(0, 0, 0.38, 0.9);
	}

	.fill {
		width: 100%;
		max-width: none;
	}

	.md {
		min-height: var(--cds-size-medium);
	}

	/* Carbon's 2xl, as in dialog footers: 64px with the label at the top. */
	.xl {
		align-items: flex-start;
		min-height: var(--cds-size-xlarge);
		padding-block: 0.875rem var(--cds-spacing-07);
	}

	.sm {
		min-height: var(--cds-size-small);
		padding-inline: calc(var(--cds-spacing-05) - 1px) calc(var(--cds-spacing-07) - 1px);
	}

	.button:focus-visible {
		outline: none;
		border-color: var(--cds-focus);
		box-shadow:
			inset 0 0 0 1px var(--cds-focus),
			inset 0 0 0 2px var(--cds-background);
	}

	.primary {
		background: var(--cds-button-primary);
		color: var(--cds-text-on-color);
	}
	.primary:hover {
		background: var(--cds-button-primary-hover);
		color: var(--cds-text-on-color);
	}
	.primary:active {
		background: var(--cds-button-primary-active);
	}

	.secondary {
		background: var(--cds-button-secondary);
		color: var(--cds-text-on-color);
	}
	.secondary:hover {
		background: var(--cds-button-secondary-hover);
		color: var(--cds-text-on-color);
	}
	.secondary:active {
		background: var(--cds-button-secondary-active);
	}

	.tertiary {
		background: transparent;
		border-color: var(--cds-button-tertiary);
		color: var(--cds-button-tertiary);
	}
	.tertiary:hover {
		background: var(--cds-button-tertiary-hover);
		color: var(--cds-text-inverse);
	}
	.tertiary:active {
		background: var(--cds-button-tertiary-active);
		color: var(--cds-text-inverse);
	}
	.tertiary:focus-visible {
		background: var(--cds-button-tertiary);
		color: var(--cds-text-inverse);
	}

	.ghost {
		background: transparent;
		color: var(--cds-link-primary);
		padding-inline-end: calc(var(--cds-spacing-05) - 1px);
	}
	.ghost:hover {
		background: var(--cds-background-hover);
		color: var(--cds-link-primary-hover);
	}
	.ghost:active {
		background: var(--cds-background-active);
	}

	.danger {
		background: var(--cds-button-danger-primary);
		color: var(--cds-text-on-color);
	}
	.danger:hover {
		background: var(--cds-button-danger-hover);
		color: var(--cds-text-on-color);
	}
	.danger:active {
		background: var(--cds-button-danger-active);
	}

	/* A destructive action next to others, such as "Delete user". */
	.danger-tertiary {
		background: transparent;
		border-color: var(--cds-button-danger-secondary);
		color: var(--cds-button-danger-secondary);
	}
	.danger-tertiary:hover {
		background: var(--cds-button-danger-hover);
		border-color: var(--cds-button-danger-hover);
		color: var(--cds-text-on-color);
	}
	.danger-tertiary:active {
		background: var(--cds-button-danger-active);
		color: var(--cds-text-on-color);
	}
	.danger-tertiary:focus-visible {
		background: var(--cds-button-danger-primary);
		color: var(--cds-text-on-color);
	}

	.button:disabled,
	.button:disabled:hover {
		background: var(--cds-button-disabled);
		border-color: var(--cds-button-disabled);
		color: var(--cds-text-on-color-disabled);
		cursor: not-allowed;
	}

	.ghost:disabled,
	.ghost:disabled:hover,
	.tertiary:disabled,
	.tertiary:disabled:hover {
		background: transparent;
		color: var(--cds-text-disabled);
	}
	.ghost:disabled {
		border-color: transparent;
	}
</style>
