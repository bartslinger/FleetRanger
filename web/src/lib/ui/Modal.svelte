<!--
	@component
	Carbon's small transactional modal around a form, opened by its own trigger
	button: "New pilot" opens "New pilot" with Cancel and Create, and "Archive
	user" opens a confirmation with Cancel and Archive user. The form posts
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
	import { page } from '$app/state';
	import Button from './Button.svelte';
	import Icon, { type IconDescriptor } from './Icon.svelte';
	import IconButton from './IconButton.svelte';
	import InlineNotification from './InlineNotification.svelte';

	interface Props {
		/** The trigger's label, which is also the title. */
		title: string;
		/** The trigger's style: primary for "New …", tertiary as a page action,
		 * danger-ghost in a table row. */
		triggerKind?: 'primary' | 'tertiary' | 'danger-tertiary' | 'danger-ghost';
		/** The trigger's accessible name, when its text alone doesn't say what
		 * it acts on, e.g. "Disconnect Claude" for each row's "Disconnect".
		 * It starts with the visible text. */
		triggerLabel?: string;
		/** Carbon's optional label above the title: what the dialog is about,
		 * e.g. the item a row's action affects. */
		label?: string;
		/** An icon-only trigger instead, e.g. Edit beside a value; the title is
		 * its label and tooltip, and `triggerLabel` its name where it repeats. */
		triggerIcon?: IconDescriptor;
		/** The icon trigger's size: md in a table row, as other row actions. */
		triggerSize?: 'xs' | 'md';
		/** The primary button, e.g. "Create". */
		submitLabel: string;
		/** Carbon's danger modal, for actions that can't be undone. */
		danger?: boolean;
		/** The form action, e.g. "?/create". */
		action: string;
		/** The backend's message for a rejected change. */
		error?: string;
		/** After a successful submit, e.g. to show a toast. */
		onsuccess?: (data: Record<string, unknown> | undefined) => void;
		/** Without its own trigger, e.g. opened from an overflow menu's item:
		 * the opener sets `open`, and focus returns to `returnFocus`. */
		trigger?: boolean;
		open?: boolean;
		returnFocus?: () => HTMLElement | undefined;
		children: Snippet;
	}

	let {
		title,
		triggerKind = 'primary',
		triggerLabel,
		label,
		triggerIcon,
		triggerSize = 'xs',
		submitLabel,
		danger = false,
		action,
		error,
		onsuccess,
		trigger = true,
		open = $bindable(false),
		returnFocus,
		children
	}: Props = $props();

	// "?/create" alone would replace the page's query, and SvelteKit lands on
	// the page without it after the action: a filtered list lost its filters.
	// The action goes after the query instead: "?session=none&/create".
	const target = $derived(
		action.startsWith('?/') && page.url.search ? `${page.url.search}&${action.slice(1)}` : action
	);

	let busy = $state(false);
	// The page's dialogs share one form result: `error` is this dialog's
	// only after its own submit was refused since it opened.
	let refused = $state(false);
	$effect(() => {
		if (open) refused = false;
	});
	let body: HTMLElement | undefined = $state();
	let footer: HTMLElement | undefined = $state();
	const submit = $derived(footer?.querySelector<HTMLElement>('[type=submit]'));
</script>

<Dialog.Root bind:open>
	{#if trigger}
		<Dialog.Trigger>
			{#snippet child({ props })}
				{#if triggerIcon}
					<IconButton
						{...props}
						icon={triggerIcon}
						label={title}
						name={triggerLabel}
						size={triggerSize}
					/>
				{:else}
					<Button {...props} kind={triggerKind} aria-label={triggerLabel}>{title}</Button>
				{/if}
			{/snippet}
		</Dialog.Trigger>
	{/if}
	<Dialog.Portal>
		<Dialog.Overlay class="modal-overlay" />
		<Dialog.Content
			class="modal layer-01"
			onCloseAutoFocus={(event) => {
				const to = returnFocus?.();
				if (!to) return;
				event.preventDefault();
				to.focus();
			}}
			onOpenAutoFocus={(event) => {
				// The first visible field, or for a confirmation the primary button.
				event.preventDefault();
				const field = body?.querySelector<HTMLElement>(
					'input:not([type=hidden]), select, textarea'
				);
				(field ?? submit)?.focus();
			}}
		>
			<form
				method="POST"
				action={target}
				use:enhance={() => {
					busy = true;
					return async ({ result, update }) => {
						busy = false;
						await update();
						refused = result.type === 'failure';
						// A redirect, e.g. to the list after deleting, is a success too.
						if (result.type === 'success' || result.type === 'redirect') {
							open = false;
							onsuccess?.(result.type === 'success' ? result.data : undefined);
						}
					};
				}}
			>
				<div class="header">
					<div>
						{#if label}<p class="modal-label">{label}</p>{/if}
						<Dialog.Title class="modal-title">{title}</Dialog.Title>
					</div>
					<!-- type="button": a button in a form submits it by default. -->
					<Dialog.Close class="modal-close" aria-label="Close" type="button">
						<Icon icon={Close} />
					</Dialog.Close>
				</div>
				<div class="body" bind:this={body}>
					{#if refused && error}<InlineNotification kind="error" title={error} />{/if}
					{@render children()}
				</div>
				<div class="footer" bind:this={footer}>
					<Dialog.Close>
						{#snippet child({ props })}
							<Button {...props} kind="secondary" size="xl" fill>Cancel</Button>
						{/snippet}
					</Dialog.Close>
					<Button type="submit" kind={danger ? 'danger' : 'primary'} size="xl" fill disabled={busy}>
						{submitLabel}
					</Button>
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

	/* Carbon's modal label: label-01 in the secondary text colour, 4px above the title. */
	.modal-label {
		margin: 0 0 var(--cds-spacing-02);
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
		overflow-wrap: anywhere;
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
