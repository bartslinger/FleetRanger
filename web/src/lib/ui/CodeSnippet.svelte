<!--
	@component
	Carbon's single-line code snippet: a read-only value to copy or read out,
	such as a pairing code, as wide as the form it belongs to (up to Carbon's
	768px), with a copy button that confirms with "Copied to clipboard".
-->
<script lang="ts">
	import Copy from '@carbon/icons/es/copy/16.js';
	import IconButton from './IconButton.svelte';

	interface Props {
		code: string;
		/** What the code is, for the copy button's name, e.g. "pairing code". */
		label: string;
	}

	let { code, label }: Props = $props();

	/** How long the button confirms the copy, as Carbon's feedback tooltip. */
	const COPIED_MS = 2000;
	const COPIED = 'Copied to clipboard';
	let copied = $state(false);

	async function copy() {
		await navigator.clipboard.writeText(code);
		copied = true;
		setTimeout(() => (copied = false), COPIED_MS);
	}
</script>

<div class="snippet">
	<code>{code}</code>
	<IconButton
		icon={Copy}
		label={copied ? COPIED : 'Copy'}
		name={copied ? COPIED : `Copy ${label}`}
		size="md"
		onclick={copy}
	/>
	<span class="visually-hidden" aria-live="polite">{copied ? COPIED : ''}</span>
</div>

<style>
	.snippet {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		max-width: 48rem;
		min-height: 2.5rem;
		background: var(--cds-layer-01);
	}

	/* code-02 rather than Carbon's code-01: the code is read out and typed
	   on another device, so it's a size larger on purpose. */
	code {
		overflow-x: auto;
		padding: 0 var(--cds-spacing-05);
		font-family: var(--cds-font-family-mono);
		font-size: var(--cds-code-02-font-size);
		font-weight: var(--cds-code-02-font-weight);
		line-height: var(--cds-code-02-line-height);
		letter-spacing: var(--cds-code-02-letter-spacing);
		white-space: nowrap;
	}
</style>
