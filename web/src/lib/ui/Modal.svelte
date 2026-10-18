<!--
	@component
	Carbon's small transactional modal around a form, opened by its own trigger
	button: "New pilot" opens "New pilot" with Cancel and Create. The form posts
	to a SvelteKit form action. On success the modal closes and the page data
	reloads; on a rejected change it stays open and shows `error`.

	Bits UI's dialog traps focus, closes on Escape and returns focus to the
	trigger. The first field gets focus when it opens, as Carbon specifies.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Dialog } from 'bits-ui';
	import Close from '@carbon/icons/es/close/20.js';
	import { enhance } from '$app/forms';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';
	import InlineNotification from './InlineNotification.svelte';

	interface Props {
		/** The trigger's label, which is also the title. */
		title: string;
		/** The primary button, e.g. "Create". */
		submitLabel: string;
		/** The form action, e.g. "?/create". */
		action: string;
		/** The backend's message for a rejected change. */
		error?: string;
		children: Snippet;
	}

	let { title, submitLabel, action, error, children }: Props = $props();

	let open = $state(false);
	let busy = $state(false);
	let body: HTMLElement | undefined = $state();
</script>

<Dialog.Root bind:open>
	<Dialog.Trigger>
		{#snippet child({ props })}
			<Button {...props}>{title}</Button>
		{/snippet}
	</Dialog.Trigger>
	<Dialog.Portal>
		<Dialog.Overlay class="modal-overlay" />
		<Dialog.Content
			class="modal layer-01"
			onOpenAutoFocus={(event) => {
				event.preventDefault();
				body?.querySelector<HTMLElement>('input, select, textarea')?.focus();
			}}
		>
			<form
				method="POST"
				{action}
				use:enhance={() => {
					busy = true;
					return async ({ result, update }) => {
						busy = false;
						if (result.type === 'success') open = false;
						await update();
					};
				}}
			>
				<div class="header">
					<Dialog.Title class="modal-title">{title}</Dialog.Title>
					<Dialog.Close class="modal-close" aria-label="Close">
						<Icon icon={Close} />
					</Dialog.Close>
				</div>
				<div class="body" bind:this={body}>
					{#if error}<InlineNotification kind="error" title={error} />{/if}
					{@render children()}
				</div>
				<div class="footer">
					<Dialog.Close>
						{#snippet child({ props })}
							<Button {...props} kind="secondary" size="xl" fill>Cancel</Button>
						{/snippet}
					</Dialog.Close>
					<Button type="submit" size="xl" fill disabled={busy}>{submitLabel}</Button>
				</div>
			</form>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>

<style>
	/* Bits UI renders these in a portal, so they're styled globally by class. */
	:global(.modal-overlay) {
		position: fixed;
		inset: 0;
		z-index: 8;
		background: var(--cds-overlay);
		animation: modal-fade 240ms cubic-bezier(0, 0, 0.3, 1);
	}

	/* Carbon's small modal: 100% wide below md and pinned to the top, then 60%,
	   42% from lg and 36% from xlg, centred. */
	:global(.modal) {
		position: fixed;
		inset: 0 0 auto;
		z-index: 9;
		display: flex;
		width: 100%;
		max-height: 100%;
		background: var(--cds-layer-01);
		border: 1px solid var(--cds-border-subtle-01);
		color: var(--cds-text-primary);
		animation: modal-in 240ms cubic-bezier(0, 0, 0.3, 1);
	}

	@media (min-width: 42rem) {
		:global(.modal) {
			inset: 50% auto auto 50%;
			width: 60%;
			max-height: 72%;
			translate: -50% -50%;
		}
	}

	@media (min-width: 66rem) {
		:global(.modal) {
			width: 42%;
		}
	}

	@media (min-width: 82rem) {
		:global(.modal) {
			width: 36%;
		}
	}

	form {
		display: grid;
		grid-template-rows: auto 1fr auto;
		width: 100%;
		min-height: 0;
	}

	.header {
		position: relative;
		margin-bottom: var(--cds-spacing-03);
		padding: var(--cds-spacing-05) var(--cds-spacing-09) 0 var(--cds-spacing-05);
	}

	:global(.modal-title) {
		margin: 0;
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	:global(.modal-close) {
		position: absolute;
		top: 0;
		right: 0;
		display: grid;
		place-items: center;
		width: var(--cds-size-large);
		height: var(--cds-size-large);
		padding: 0;
		border: 2px solid transparent;
		background: none;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	:global(.modal-close:hover) {
		background: var(--cds-layer-hover-01);
	}

	:global(.modal-close:focus-visible) {
		outline: none;
		border-color: var(--cds-focus);
	}

	/* Fields 24px apart, and 48px from the last one to the buttons. */
	.body {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-06);
		padding: var(--cds-spacing-03) var(--cds-spacing-05) var(--cds-spacing-09);
		overflow-y: auto;
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
	}

	.footer {
		display: grid;
		grid-template-columns: 1fr 1fr;
	}

	@keyframes modal-fade {
		from {
			opacity: 0;
		}
	}

	@keyframes modal-in {
		from {
			opacity: 0;
			transform: translateY(-1.5rem);
		}
	}
</style>
