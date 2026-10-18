<!--
	@component
	Carbon's UI shell: a header, side navigation and the page content. Below the
	lg breakpoint the navigation is hidden behind a menu button.
-->
<script lang="ts" module>
	export interface NavItem {
		/** Already passed through `resolve()`. */
		href: string;
		label: string;
	}
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';

	interface Props {
		name: string;
		nav: NavItem[];
		children: Snippet;
	}

	let { name, nav, children }: Props = $props();

	let open = $state(false);

	afterNavigate(() => (open = false));

	const home = resolve('/');

	function isCurrent(href: string): boolean {
		// resolve() can return relative paths (`./`), so compare absolute ones.
		const absolute = (path: string) => new URL(path, page.url.href).pathname.replace(/\/$/, '');
		const [path, target] = [absolute(page.url.pathname), absolute(href)];
		return href === home ? path === target : path === target || path.startsWith(`${target}/`);
	}
</script>

<svelte:window onkeydown={(event) => event.key === 'Escape' && (open = false)} />

<header>
	<button
		class="menu"
		type="button"
		aria-label={open ? 'Close menu' : 'Open menu'}
		aria-expanded={open}
		aria-controls="side-nav"
		onclick={() => (open = !open)}
	>
		<!-- Carbon icons "menu" and "close" (Apache-2.0). -->
		<svg viewBox="0 0 32 32" width="20" height="20" aria-hidden="true" fill="currentColor">
			{#if open}
				<path
					d="M17.4141 16L24 9.4141 22.5859 8 16 14.5859 9.4143 8 8 9.4141 14.5859 16 8 22.5859 9.4143 24 16 17.4141 22.5859 24 24 22.5859 17.4141 16z"
				/>
			{:else}
				<path d="M4 6H28V8H4zM4 24H28V26H4zM4 12H28V14H4zM4 18H28V20H4z" />
			{/if}
		</svg>
	</button>
	<a class="name" href={home}>{name}</a>
</header>

<nav id="side-nav" class:open aria-label="Main">
	<ul>
		{#each nav as item (item.href)}
			<li>
				<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller -->
				<a href={item.href} aria-current={isCurrent(item.href) ? 'page' : undefined}>
					{item.label}
				</a>
			</li>
		{/each}
	</ul>
</nav>

{#if open}
	<button
		class="scrim"
		type="button"
		tabindex="-1"
		aria-hidden="true"
		onclick={() => (open = false)}
	></button>
{/if}

<main>
	{@render children()}
</main>

<style>
	header {
		position: fixed;
		inset: 0 0 auto;
		z-index: 3;
		display: flex;
		align-items: center;
		height: var(--cds-size-large);
		background: var(--cds-background);
		border-bottom: 1px solid var(--cds-border-subtle-01);
	}

	.menu {
		display: grid;
		place-items: center;
		width: var(--cds-size-large);
		height: 100%;
		padding: 0;
		border: 0;
		background: none;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	.menu:hover {
		background: var(--cds-background-hover);
	}

	.name {
		display: flex;
		align-items: center;
		height: 100%;
		padding: 0 var(--cds-spacing-05);
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
		text-decoration: none;
	}

	nav {
		position: fixed;
		top: var(--cds-size-large);
		bottom: 0;
		left: 0;
		z-index: 2;
		width: 16rem;
		padding-top: var(--cds-spacing-05);
		overflow-y: auto;
		background: var(--cds-background);
		/* Hidden, not just moved, so closed links can't be focused. */
		visibility: hidden;
		transform: translateX(-100%);
		transition:
			transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9),
			visibility 110ms;
	}

	nav.open {
		visibility: visible;
		transform: none;
	}

	nav a {
		position: relative;
		display: flex;
		align-items: center;
		min-height: var(--cds-size-small);
		padding: 0 var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
		text-decoration: none;
	}

	nav a:hover {
		background: var(--cds-background-hover);
		color: var(--cds-text-primary);
	}

	nav a[aria-current='page'] {
		background: var(--cds-background-selected);
		color: var(--cds-text-primary);
	}

	nav a[aria-current='page']::before {
		content: '';
		position: absolute;
		inset: 0 auto 0 0;
		width: 3px;
		background: var(--cds-border-interactive);
	}

	.scrim {
		position: fixed;
		inset: var(--cds-size-large) 0 0;
		z-index: 1;
		padding: 0;
		border: 0;
		background: var(--cds-overlay);
	}

	/* Carbon pads the content by 2rem; phones get 1rem to keep room for content. */
	main {
		padding: calc(var(--cds-size-large) + var(--cds-spacing-07)) var(--cds-spacing-05)
			var(--cds-spacing-07);
	}

	/* Carbon's lg breakpoint (66rem): navigation always visible. Custom properties
	   can't be used in media queries. */
	@media (min-width: 66rem) {
		.menu,
		.scrim {
			display: none;
		}

		nav {
			visibility: visible;
			transform: none;
			transition: none;
		}

		.name {
			padding-left: var(--cds-spacing-05);
		}

		main {
			margin-left: 16rem;
			padding-inline: var(--cds-spacing-07);
		}
	}
</style>
