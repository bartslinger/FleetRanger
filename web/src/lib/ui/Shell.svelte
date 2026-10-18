<!--
	@component
	Carbon's UI shell: a header, side navigation and the page content, inline
	themed dark. From the lg breakpoint the navigation is Carbon's rail: icons
	only, expanding over the content on hover or focus. Below lg it is hidden
	behind a menu button. Without an account (the login page) the header holds
	only the product name.
-->
<script lang="ts" module>
	import type { IconDescriptor } from './Icon.svelte';

	export interface NavItem {
		/** Already passed through `resolve()`. */
		href: string;
		label: string;
		/** A 16px icon from @carbon/icons. */
		icon: IconDescriptor;
	}

	export interface ShellAccount {
		email: string;
		name: string | null;
	}
</script>

<script lang="ts">
	import { tick, type Snippet } from 'svelte';
	import UserAvatar from '@carbon/icons/es/user--avatar/20.js';
	import { afterNavigate } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { setTheme, type Theme } from '#lib/theme.js';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';
	import RadioButtonGroup from './RadioButtonGroup.svelte';

	interface Props {
		name: string;
		nav: NavItem[];
		/** Who is logged in; null before logging in. */
		account: ShellAccount | null;
		/** The user's theme choice as rendered by the server. */
		theme: Theme;
		children: Snippet;
	}

	let { name, nav, account, theme: initialTheme, children }: Props = $props();

	let open = $state(false);
	let menuButton: HTMLButtonElement | undefined = $state();
	let navElement: HTMLElement | undefined = $state();

	// The rail (lg and up) expands after Carbon's 100ms delay, so passing over
	// it with the mouse doesn't flash the labels, and collapses at once.
	let expanded = $state(false);
	let expandTimer: ReturnType<typeof setTimeout> | undefined;

	function expandSoon() {
		clearTimeout(expandTimer);
		expandTimer = setTimeout(() => (expanded = true), 100);
	}

	function collapse() {
		clearTimeout(expandTimer);
		expanded = false;
	}

	function onNavFocusout(event: FocusEvent) {
		if (!navElement?.contains(event.relatedTarget as Node | null)) collapse();
	}

	// The profile panel: who is logged in, the theme and logging out. The theme
	// starts from the server's value, then follows the user's choices.
	// svelte-ignore state_referenced_locally
	let theme = $state(initialTheme);
	let panelOpen = $state(false);
	let panelButton: HTMLButtonElement | undefined = $state();
	let panelElement: HTMLElement | undefined = $state();

	afterNavigate(() => {
		open = false;
		panelOpen = false;
	});

	async function toggle() {
		open = !open;
		if (open) {
			// One panel at a time.
			panelOpen = false;
			// Move focus into the navigation, as Carbon's left panel does.
			await tick();
			navElement?.querySelector('a')?.focus();
		}
	}

	async function togglePanel() {
		panelOpen = !panelOpen;
		if (panelOpen) {
			open = false;
			// Focus the checked option, as in Carbon's settings menus.
			await tick();
			panelElement?.querySelector<HTMLInputElement>('input:checked')?.focus();
		}
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Escape') return;
		if (panelOpen) {
			panelOpen = false;
			panelButton?.focus();
		} else if (open) {
			open = false;
			menuButton?.focus();
		} else if (expanded) {
			// As Carbon's rail: collapse and continue in the content.
			collapse();
			document.getElementById('main-content')?.focus();
		}
	}

	/** Closes the panel on a click outside it or its button. */
	function onclick(event: MouseEvent) {
		const target = event.target as Node;
		if (panelOpen && !panelElement?.contains(target) && !panelButton?.contains(target)) {
			panelOpen = false;
		}
	}

	/** Closes the panel when focus leaves it, e.g. by tabbing past the end. */
	function onfocusout(event: FocusEvent) {
		const next = event.relatedTarget as Node | null;
		if (next && !panelElement?.contains(next) && !panelButton?.contains(next)) {
			panelOpen = false;
		}
	}

	const home = resolve('/');

	function isCurrent(href: string): boolean {
		// resolve() can return relative paths (`./`), so compare absolute ones.
		const absolute = (path: string) => new URL(path, page.url.href).pathname.replace(/\/$/, '');
		const [path, target] = [absolute(page.url.pathname), absolute(href)];
		return href === home ? path === target : path === target || path.startsWith(`${target}/`);
	}
</script>

<svelte:window {onkeydown} {onclick} />

<a class="skip" href="#main-content">Skip to main content</a>

<header class="theme-dark">
	{#if account}
		<button
			bind:this={menuButton}
			class="menu"
			type="button"
			aria-label={open ? 'Close menu' : 'Open menu'}
			aria-expanded={open}
			aria-controls="side-nav"
			onclick={toggle}
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
	{/if}
	<a class="name" href={home}>{name}</a>
	{#if account}
		<div class="utilities">
			<button
				bind:this={panelButton}
				class="action"
				class:active={panelOpen}
				type="button"
				aria-label="Profile"
				aria-expanded={panelOpen}
				aria-controls="profile-panel"
				onclick={togglePanel}
				{onfocusout}
			>
				<Icon icon={UserAvatar} />
				<span class="tooltip" aria-hidden="true">Profile</span>
			</button>
			{#if panelOpen}
				<section
					bind:this={panelElement}
					id="profile-panel"
					class="panel"
					aria-label="Profile"
					{onfocusout}
				>
					<div class="identity">
						{#if account.name}<p class="account-name">{account.name}</p>{/if}
						<p class="email">{account.email}</p>
					</div>
					<RadioButtonGroup
						legend="Theme"
						name="theme"
						options={[
							{ value: 'system', label: 'System' },
							{ value: 'light', label: 'Light' },
							{ value: 'dark', label: 'Dark' }
						]}
						bind:value={theme}
						onchange={setTheme}
					/>
					<!-- Posted straight to the backend, which ends the login session. -->
					<form class="logout" method="post" action="/auth/logout">
						<Button kind="ghost" type="submit" fill>Log out</Button>
					</form>
				</section>
			{/if}
		</div>
	{/if}
</header>

{#if account}
	<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
	<!-- A click expands the rail at once; keyboard users expand it by focus. -->
	<nav
		bind:this={navElement}
		id="side-nav"
		class="theme-dark"
		class:open
		class:expanded
		aria-label="Main"
		onmouseenter={expandSoon}
		onmouseleave={collapse}
		onfocusin={expandSoon}
		onfocusout={onNavFocusout}
		onclick={() => (expanded = true)}
	>
		<ul>
			{#each nav as item (item.href)}
				<li>
					<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller -->
					<a href={item.href} aria-current={isCurrent(item.href) ? 'page' : undefined}>
						<Icon icon={item.icon} />
						<span>{item.label}</span>
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
{/if}

<main id="main-content" tabindex="-1" class:bare={!account}>
	{@render children()}
</main>

<style>
	/* Carbon's skip link: hidden until focused, then shown over the header. */
	.skip {
		position: absolute;
		top: 0;
		left: 0;
		z-index: 4;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}

	.skip:focus {
		display: flex;
		align-items: center;
		width: auto;
		height: var(--cds-size-large);
		padding: 0 var(--cds-spacing-05);
		clip-path: none;
		outline: none;
		border: 4px solid var(--cds-focus);
		background: var(--cds-background);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		text-decoration: none;
	}

	main:focus {
		outline: none;
	}

	header {
		position: fixed;
		inset: 0 0 auto;
		z-index: 3;
		display: flex;
		align-items: center;
		height: var(--cds-size-large);
		background: var(--cds-background);
		border-bottom: 1px solid var(--cds-border-subtle-00);
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

	.menu:active {
		background: var(--cds-background-active);
	}

	/* Carbon's header name: body-compact-01 in semibold, close to the menu
	   button while that is shown. */
	.name {
		display: flex;
		align-items: center;
		height: 100%;
		padding: 0 var(--cds-spacing-07) 0 var(--cds-spacing-03);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		font-weight: 600;
		line-height: 1.25rem;
		letter-spacing: 0.1px;
		text-decoration: none;
	}

	/* Header utilities: Carbon's header global actions, at the right edge. */
	.utilities {
		display: flex;
		height: 100%;
		margin-left: auto;
	}

	.action {
		position: relative;
		display: grid;
		place-items: center;
		width: var(--cds-size-large);
		height: 100%;
		padding: 0;
		border: 1px solid transparent;
		background: none;
		color: var(--cds-icon-secondary);
		cursor: pointer;
		transition:
			background-color 110ms,
			border-color 110ms;
	}

	.action:hover {
		background: var(--cds-background-hover);
		color: var(--cds-icon-primary);
	}

	.action:active {
		background: var(--cds-background-active);
	}

	.action:focus-visible {
		outline: none;
		border-color: var(--cds-focus);
	}

	/* Open: outlined, with the bottom edge flowing into the panel. */
	.action.active {
		border-color: var(--cds-border-subtle-00);
		border-bottom-color: transparent;
		background: var(--cds-layer-01);
		color: var(--cds-icon-primary);
	}

	.tooltip {
		position: absolute;
		top: calc(100% + var(--cds-spacing-02));
		right: 0;
		padding: var(--cds-spacing-01) var(--cds-spacing-05);
		background: var(--cds-background-inverse);
		color: var(--cds-text-inverse);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		white-space: nowrap;
		visibility: hidden;
		pointer-events: none;
	}

	.action:not(.active):hover .tooltip,
	.action:not(.active):focus-visible .tooltip {
		visibility: visible;
	}

	.panel {
		position: fixed;
		top: var(--cds-size-large);
		right: 0;
		bottom: 0;
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-06);
		width: 16rem;
		padding: var(--cds-spacing-05);
		overflow-y: auto;
		background: var(--cds-layer-01);
		border-inline: 1px solid var(--cds-border-subtle-00);
		animation: panel-in 110ms cubic-bezier(0, 0, 0.38, 0.9);
	}

	.identity p {
		margin: 0;
		overflow-wrap: anywhere;
	}

	.account-name {
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.email {
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	/* Full width, below a divider, as the last item of the panel. */
	.logout {
		margin-inline: calc(-1 * var(--cds-spacing-05));
		border-top: 1px solid var(--cds-border-subtle-00);
	}

	@keyframes panel-in {
		from {
			transform: translateX(100%);
		}
	}

	nav {
		position: fixed;
		top: var(--cds-size-large);
		bottom: 0;
		left: 0;
		z-index: 2;
		width: 16rem;
		padding-top: var(--cds-spacing-05);
		overflow: hidden auto;
		background: var(--cds-background);
		/* Hidden, not just moved, so closed links can't be focused. */
		visibility: hidden;
		transform: translateX(-100%);
		transition:
			transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9),
			visibility 110ms;
	}

	/* Visible at once when opening, so focus can move into it; the delay
	   above only applies when closing. */
	nav.open {
		visibility: visible;
		transform: none;
		transition:
			transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9),
			visibility 0s;
	}

	/* Carbon's side nav link: a 16px icon, then the label 24px further. In the
	   collapsed rail the label is clipped, not removed, so it still names the
	   link for screen readers. */
	nav a {
		position: relative;
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-06);
		min-height: var(--cds-size-small);
		padding: 0 var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
		text-decoration: none;
		white-space: nowrap;
	}

	nav a :global(svg) {
		flex: none;
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

	/* No side padding: pages lay out their content on the Grid, which has the
	   margins. */
	main {
		padding: var(--cds-size-large) 0 var(--cds-spacing-09);
	}

	/* Carbon's lg breakpoint (66rem): the rail, always visible, 48px wide and
	   256px when expanded over the content. Custom properties can't be used in
	   media queries. */
	@media (min-width: 66rem) {
		.menu,
		.scrim {
			display: none;
		}

		nav {
			width: var(--cds-size-large);
			visibility: visible;
			transform: none;
			transition: width 110ms cubic-bezier(0.2, 0, 0.38, 0.9);
		}

		nav.expanded {
			width: 16rem;
		}

		.name {
			padding-left: var(--cds-spacing-05);
		}

		main:not(.bare) {
			margin-left: var(--cds-size-large);
		}
	}
</style>
