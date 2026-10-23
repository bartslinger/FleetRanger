<!--
	@component
	Carbon's single-line code snippet: a read-only value to copy or read out,
	such as a pairing code, with a copy button that confirms with "Copied".
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

	/** How long the button says "Copied", as Carbon's feedback tooltip. */
	const COPIED_MS = 2000;
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
		label={copied ? 'Copied' : 'Copy'}
		name={copied ? 'Copied' : `Copy ${label}`}
		size="md"
		onclick={copy}
	/>
	<span class="visually-hidden" aria-live="polite">{copied ? 'Copied' : ''}</span>
</div>

<style>
	.snippet {
		display: inline-flex;
		align-items: center;
		max-width: 100%;
		min-height: 2.5rem;
		background: var(--cds-layer-01);
	}

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
