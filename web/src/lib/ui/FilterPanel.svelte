<!--
	@component
	Filters behind a button, in a panel from the right (full screen on
	phones): Carbon's batch filtering. The fields change a draft; Apply applies
	it, Reset empties the fields. Closing without Apply discards the draft, so
	reopening shows what's applied. The button says how many filters are
	applied.

	Bits UI's dialog traps focus, closes on Escape and returns focus to the
	button. The first field gets focus when it opens.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Dialog } from 'bits-ui';
	import Close from '@carbon/icons/es/close/20.js';
	import Filter from '@carbon/icons/es/filter/16.js';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		/** How many filters are applied, shown on the button. */
		count: number;
		/** Before it opens: the draft starts as what's applied. */
		onopen: () => void;
		/** Applies the draft. */
		onapply: () => void;
		/** Empties the draft's fields. */
		onreset: () => void;
		/** The button's: lg in a table's toolbar, md beside fields. */
		size?: 'md' | 'lg';
		children: Snippet;
	}

	let { count, onopen, onapply, onreset, size = 'lg', children }: Props = $props();

	let open = $state(false);
	let body: HTMLElement | undefined = $state();
</script>

<Dialog.Root
	bind:open
	onOpenChange={(next) => {
		if (next) onopen();
	}}
>
	<Dialog.Trigger>
		{#snippet child({ props })}
			<Button {...props} kind="ghost" {size} icon={Filter}>
				Filters{count > 0 ? ` (${count})` : ''}
			</Button>
		{/snippet}
	</Dialog.Trigger>
	<Dialog.Portal>
		<Dialog.Overlay class="modal-overlay" />
		<Dialog.Content
			class="filter-panel layer-01"
			onOpenAutoFocus={(event) => {
				event.preventDefault();
				body?.querySelector<HTMLElement>('input:not([type=hidden]), select, button')?.focus();
			}}
		>
			<div class="header">
				<Dialog.Title class="modal-title">Filters</Dialog.Title>
				<Dialog.Close class="modal-close" aria-label="Close">
					<Icon icon={Close} />
				</Dialog.Close>
			</div>
			<div class="body" bind:this={body}>
				{@render children()}
			</div>
			<div class="footer">
				<Button kind="secondary" size="xl" fill onclick={onreset}>Reset</Button>
				<Button
					kind="primary"
					size="xl"
					fill
					onclick={() => {
						onapply();
						open = false;
					}}>Apply</Button
				>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>

<style>
	/* Over the page from the right, full height: 20rem from md, the whole
	   screen below. The overlay, title and close button are the modal's. */
	:global(.filter-panel) {
		position: fixed;
		inset: 0;
		z-index: 9;
		display: grid;
		grid-template-rows: auto 1fr auto;
		background: var(--cds-layer-01);
		color: var(--cds-text-primary);
		animation: filter-panel-in 240ms cubic-bezier(0, 0, 0.3, 1);
	}

	@media (min-width: 42rem) {
		:global(.filter-panel) {
			inset: 0 0 0 auto;
			width: 20rem;
			border-left: 1px solid var(--cds-border-subtle-01);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		:global(.filter-panel) {
			animation: none;
		}
	}

	.header {
		position: relative;
		padding: var(--cds-spacing-05) var(--cds-spacing-09) var(--cds-spacing-05) var(--cds-spacing-05);
	}

	/* Fields 24px apart, as in a form. */
	.body {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-06);
		min-height: 0;
		padding: var(--cds-spacing-03) var(--cds-spacing-05) var(--cds-spacing-07);
		overflow-y: auto;
	}

	.footer {
		display: grid;
		grid-template-columns: 1fr 1fr;
	}

	@keyframes filter-panel-in {
		from {
			opacity: 0;
			transform: translateX(2rem);
		}
	}
</style>
