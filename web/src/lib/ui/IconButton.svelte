<!--
	@component
	Carbon's icon-only ghost button: an icon, its label as the accessible name
	and in a tooltip on hover and focus (Carbon requires one for every
	icon-only button). Extra small (24px) beside a value, small (32px) on
	phones for touch. Other props (e.g. a dialog trigger's) go to the button.
-->
<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { mergeProps, Tooltip } from 'bits-ui';
	import Icon, { type IconDescriptor } from './Icon.svelte';

	interface Props extends HTMLButtonAttributes {
		icon: IconDescriptor;
		/** What it does, e.g. "Change pilot". */
		label: string;
	}

	let { icon, label, ...rest }: Props = $props();
</script>

<Tooltip.Provider delayDuration={100}>
	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<button {...mergeProps(rest, props)} type="button" class="icon-button" aria-label={label}>
					<Icon {icon} />
				</button>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Portal>
			<Tooltip.Content class="tooltip" side="bottom" sideOffset={4} collisionPadding={16}>
				{label}
			</Tooltip.Content>
		</Tooltip.Portal>
	</Tooltip.Root>
</Tooltip.Provider>

<style>
	.icon-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--cds-spacing-06);
		height: var(--cds-spacing-06);
		margin: 0;
		padding: 0;
		border: 1px solid transparent;
		border-radius: 0;
		background: transparent;
		color: var(--cds-icon-primary);
		vertical-align: middle;
		cursor: pointer;
		transition: background 70ms cubic-bezier(0, 0, 0.38, 0.9);
	}

	/* 32px on phones: a better touch target. */
	@media (max-width: 41.99rem) {
		.icon-button {
			width: var(--cds-spacing-07);
			height: var(--cds-spacing-07);
		}
	}

	.icon-button:hover {
		background: var(--cds-background-hover);
	}

	.icon-button:active {
		background: var(--cds-background-active);
	}

	.icon-button:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Carbon's tooltip: inverse, body-compact-01, rendered in a portal. */
	:global(.tooltip) {
		z-index: 10;
		max-width: 18rem;
		padding: var(--cds-spacing-02) var(--cds-spacing-05);
		background: var(--cds-background-inverse);
		color: var(--cds-text-inverse);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		border-radius: 2px;
	}
</style>
