<!--
	@component
	Carbon's content switcher (small), switching between views of the same
	data, e.g. a table and cards. The options are links, so the view is part
	of the URL.
-->
<script lang="ts">
	interface Props {
		/** E.g. "View". */
		label: string;
		options: { label: string; href: string; selected: boolean }[];
	}

	let { label, options }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- links are resolved by the caller -->
<nav class="switcher" aria-label={label}>
	{#each options as option (option.href)}
		<a
			href={option.href}
			class:selected={option.selected}
			aria-current={option.selected ? 'page' : undefined}
			data-sveltekit-reset={false}
			data-sveltekit-replacestate>{option.label}</a
		>
	{/each}
</nav>

<style>
	/* Every option as wide as the widest, as Carbon asks. */
	.switcher {
		display: inline-grid;
		grid-auto-columns: 1fr;
		grid-auto-flow: column;
	}

	a {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		height: 2rem;
		padding: 0 var(--cds-spacing-05);
		border: 1px solid var(--cds-border-inverse);
		background: transparent;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
		transition: background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	a + a {
		border-left: 0;
	}

	a:hover:not(.selected) {
		background: var(--cds-background-hover);
		color: var(--cds-text-primary);
	}

	a.selected {
		background: var(--cds-layer-selected-inverse);
		color: var(--cds-text-inverse);
		cursor: default;
	}

	a:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
		box-shadow: inset 0 0 0 3px var(--cds-focus-inset);
	}

	@media (prefers-reduced-motion: reduce) {
		a {
			transition: none;
		}
	}
</style>
