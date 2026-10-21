<!--
	@component
	Carbon's icon-only ghost button: an icon, its label as the accessible name
	and in a tooltip on hover and focus (Carbon requires one for every
	icon-only button). Extra small (24px) beside a value, small (32px) on
	phones for touch; medium (40px, 48px on phones) for a control used over
	and over. With `href` it is a link, e.g. to the next item; disabled, it
	keeps its place without a tooltip. Other props (e.g. a dialog trigger's)
	go to the button or link.
-->
<script lang="ts">
	import type { HTMLAttributes } from 'svelte/elements';
	import { mergeProps, Tooltip } from 'bits-ui';
	import Icon, { type IconDescriptor } from './Icon.svelte';
	import TooltipContent from './TooltipContent.svelte';

	interface Props extends HTMLAttributes<HTMLElement> {
		icon: IconDescriptor;
		/** What it does, e.g. "Change pilot": its tooltip. */
		label: string;
		/** A longer accessible name where buttons repeat, e.g. one per table
		 * row: "Leave out flight 9 Oct, 18:01". It starts with `label`, so
		 * voice control can say what the tooltip shows. */
		name?: string;
		/** Already passed through `resolve()`. */
		href?: string;
		size?: 'xs' | 'md';
		disabled?: boolean;
	}

	let { icon, label, name, href, size = 'xs', disabled = false, ...rest }: Props = $props();
</script>

{#if disabled}
	<span class="icon-button {size} disabled" role={href ? 'link' : 'button'} aria-disabled="true">
		<Icon {icon} label={name ?? label} />
	</span>
{:else}
	<Tooltip.Provider delayDuration={100}>
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					{#if href}
						<!-- eslint-disable svelte/no-navigation-without-resolve -- resolved by the caller -->
						<a
							{...mergeProps(rest, props)}
							{href}
							class="icon-button {size}"
							aria-label={name ?? label}
						>
							<Icon {icon} />
						</a>
						<!-- eslint-enable svelte/no-navigation-without-resolve -->
					{:else}
						<button
							{...mergeProps(rest, props)}
							type="button"
							class="icon-button {size}"
							aria-label={name ?? label}
						>
							<Icon {icon} />
						</button>
					{/if}
				{/snippet}
			</Tooltip.Trigger>
			<TooltipContent>{label}</TooltipContent>
		</Tooltip.Root>
	</Tooltip.Provider>
{/if}

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

	.md {
		width: var(--cds-spacing-08);
		height: var(--cds-spacing-08);
	}

	/* Larger on phones: a better touch target. */
	@media (max-width: 41.99rem) {
		.icon-button {
			width: var(--cds-spacing-07);
			height: var(--cds-spacing-07);
		}

		.md {
			width: var(--cds-spacing-09);
			height: var(--cds-spacing-09);
		}
	}

	.icon-button:hover:not(.disabled) {
		background: var(--cds-background-hover);
	}

	.icon-button:active:not(.disabled) {
		background: var(--cds-background-active);
	}

	.icon-button:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	.disabled {
		color: var(--cds-icon-disabled);
		cursor: not-allowed;
	}
</style>
