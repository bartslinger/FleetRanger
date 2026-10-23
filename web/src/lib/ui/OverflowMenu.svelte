<!--
	@component
	Carbon's overflow menu: an icon-only "…" button that opens a menu of
	actions, for those that don't fit or are used less. Small inline (e.g. in
	a breadcrumb), or as large as the batch action bar's buttons, on its brand
	colour. Bits UI handles the keyboard and focus. Items can be for phones
	only, e.g. an action that is a button of its own on wider screens.
-->
<script lang="ts" module>
	export interface OverflowItem {
		label: string;
		onselect: () => void;
		/** Only below md, where the action has no button of its own. */
		phoneOnly?: boolean;
		/** A destructive action, e.g. Delete: last, below a divider, in red. */
		danger?: boolean;
		/** Carbon's selectable item, an on/off setting with a checkmark while
		 * on, e.g. a toolbar's toggle that doesn't fit on phones. */
		checked?: boolean;
	}
</script>

<script lang="ts">
	import { DropdownMenu, Tooltip } from 'bits-ui';
	import TooltipContent from './TooltipContent.svelte';
	import Checkmark from '@carbon/icons/es/checkmark/16.js';
	import OverflowMenuHorizontal from '@carbon/icons/es/overflow-menu--horizontal/16.js';
	import OverflowMenuVertical from '@carbon/icons/es/overflow-menu--vertical/16.js';
	import Icon, { type IconDescriptor } from './Icon.svelte';

	interface Props {
		/** The button's accessible name, e.g. "More actions". */
		label: string;
		items: OverflowItem[];
		/** As small as text (a breadcrumb), as large as a row (48px, e.g. a
		 * contained list's), or the batch action bar's. */
		variant?: 'inline' | 'row' | 'batch';
		/** The button, e.g. to give focus back to after a dialog. */
		trigger?: HTMLElement | null;
		/** Carbon's vertical icon, as in a table toolbar. */
		vertical?: boolean;
		/** An icon that says what the menu is about, e.g. a calendar for
		 * choosing a period, in place of the overflow icon. */
		icon?: IconDescriptor;
	}

	let {
		label,
		items,
		variant = 'inline',
		trigger = $bindable(null),
		vertical = false,
		icon
	}: Props = $props();
</script>

<DropdownMenu.Root>
	<!-- Its name in a tooltip, as for every icon-only button (Carbon). -->
	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<DropdownMenu.Trigger
					{...props}
					bind:ref={trigger}
					class="overflow-trigger {variant}"
					aria-label={label}
				>
					<Icon icon={icon ?? (vertical ? OverflowMenuVertical : OverflowMenuHorizontal)} />
				</DropdownMenu.Trigger>
			{/snippet}
		</Tooltip.Trigger>
		<TooltipContent kind="icon">{label}</TooltipContent>
	</Tooltip.Root>
	<DropdownMenu.Portal>
		<DropdownMenu.Content
			class="overflow-menu"
			align={variant === 'inline' ? 'start' : 'end'}
			sideOffset={variant === 'inline' ? 4 : 0}
			collisionPadding={16}
		>
			<!-- A breadcrumb trail can pass the same page twice. -->
			{#each items as item, index (index)}
				{#if item.danger && index > 0}
					<DropdownMenu.Separator class="overflow-divider" />
				{/if}
				{#if item.checked !== undefined}
					<DropdownMenu.CheckboxItem
						class="overflow-item selectable{item.phoneOnly ? ' phone-only' : ''}"
						checked={item.checked}
						onCheckedChange={item.onselect}
					>
						<span class="check"
							>{#if item.checked}<Icon icon={Checkmark} />{/if}</span
						>
						{item.label}
					</DropdownMenu.CheckboxItem>
				{:else}
					<DropdownMenu.Item
						class="overflow-item{item.phoneOnly ? ' phone-only' : ''}{item.danger ? ' danger' : ''}"
						onSelect={item.onselect}
					>
						{item.label}
					</DropdownMenu.Item>
				{/if}
			{/each}
		</DropdownMenu.Content>
	</DropdownMenu.Portal>
</DropdownMenu.Root>

<style>
	/* The trigger: an icon-only ghost button, as high as a breadcrumb's text
	   (larger on phones, for touch), or 48px in the batch action bar. */
	:global(.overflow-trigger) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		border: 0;
		background: transparent;
		cursor: pointer;
	}

	:global(.overflow-trigger.inline) {
		width: var(--cds-spacing-06);
		height: var(--cds-spacing-05);
		color: var(--cds-icon-primary);
	}

	@media (max-width: 41.99rem) {
		:global(.overflow-trigger.inline) {
			width: var(--cds-spacing-07);
			height: var(--cds-spacing-06);
		}
	}

	/* Carbon's large icon button (48px), the height of a row. */
	:global(.overflow-trigger.row) {
		flex: none;
		width: var(--cds-size-large);
		height: var(--cds-size-large);
		color: var(--cds-icon-primary);
	}

	:global(.overflow-trigger.inline:hover),
	:global(.overflow-trigger.inline[data-state='open']),
	:global(.overflow-trigger.row:hover),
	:global(.overflow-trigger.row[data-state='open']) {
		background: var(--cds-background-hover);
	}

	:global(.overflow-trigger.inline:focus-visible),
	:global(.overflow-trigger.row:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	:global(.overflow-trigger.batch) {
		flex: none;
		width: var(--cds-size-large);
		height: var(--cds-size-large);
		color: var(--cds-icon-on-color);
	}

	:global(.overflow-trigger.batch:hover),
	:global(.overflow-trigger.batch[data-state='open']) {
		background: var(--cds-button-primary-hover);
	}

	/* White on the blue bar in both themes. */
	:global(.overflow-trigger.batch:focus-visible) {
		outline: 2px solid var(--cds-text-on-color);
		outline-offset: -2px;
	}

	/* Carbon's overflow menu, medium: 40px options on the layer, with a
	   shadow; rendered in a portal. */
	:global(.overflow-menu) {
		z-index: 10;
		min-width: 10rem;
		max-width: 20rem;
		background: var(--cds-layer-01);
		box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.3);
	}

	:global(.overflow-item) {
		display: flex;
		align-items: center;
		min-height: 2.5rem;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		cursor: pointer;
		outline: none;
	}

	:global(.overflow-item[data-highlighted]) {
		background: var(--cds-layer-hover-01);
		color: var(--cds-text-primary);
	}

	/* A selectable item keeps 16px for its checkmark, 8px before the label. */
	:global(.overflow-item.selectable) {
		gap: var(--cds-spacing-03);
	}

	.check {
		display: inline-flex;
		flex: none;
		width: 1rem;
		color: var(--cds-icon-primary);
	}

	:global(.overflow-item.danger) {
		color: var(--cds-text-error);
	}

	:global(.overflow-item.danger[data-highlighted]) {
		background: var(--cds-button-danger-primary);
		color: var(--cds-text-on-color);
	}

	:global(.overflow-divider) {
		height: 1px;
		background: var(--cds-border-subtle-01);
	}

	:global(.overflow-item:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	@media (min-width: 42rem) {
		:global(.overflow-item.phone-only) {
			display: none;
		}
	}
</style>
