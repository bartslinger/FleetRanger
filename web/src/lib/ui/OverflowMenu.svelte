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
	}
</script>

<script lang="ts">
	import { DropdownMenu } from 'bits-ui';
	import OverflowMenuHorizontal from '@carbon/icons/es/overflow-menu--horizontal/16.js';
	import Icon from './Icon.svelte';

	interface Props {
		/** The button's accessible name, e.g. "More actions". */
		label: string;
		items: OverflowItem[];
		variant?: 'inline' | 'batch';
		/** The button, e.g. to give focus back to after a dialog. */
		trigger?: HTMLElement | null;
	}

	let { label, items, variant = 'inline', trigger = $bindable(null) }: Props = $props();
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger bind:ref={trigger} class="overflow-trigger {variant}" aria-label={label}>
		<Icon icon={OverflowMenuHorizontal} />
	</DropdownMenu.Trigger>
	<DropdownMenu.Portal>
		<DropdownMenu.Content
			class="overflow-menu"
			align={variant === 'batch' ? 'end' : 'start'}
			sideOffset={variant === 'batch' ? 0 : 4}
			collisionPadding={16}
		>
			<!-- A breadcrumb trail can pass the same page twice. -->
			{#each items as item, index (index)}
				<DropdownMenu.Item
					class="overflow-item{item.phoneOnly ? ' phone-only' : ''}"
					onSelect={item.onselect}
				>
					{item.label}
				</DropdownMenu.Item>
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

	:global(.overflow-trigger.inline:hover),
	:global(.overflow-trigger.inline[data-state='open']) {
		background: var(--cds-background-hover);
	}

	:global(.overflow-trigger.inline:focus-visible) {
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
