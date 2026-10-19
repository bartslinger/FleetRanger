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

	/** A header global action left of Profile: an icon button opening a panel.
	 * Carbon's header utilities open panels; they don't go to pages. */
	export interface HeaderAction {
		id: string;
		/** The button's name, shown as its tooltip. */
		label: string;
		/** The 20px icon. */
		icon: Snippet;
		panel: Snippet;
		/** When its panel opens or closes. */
		ontoggle?: (open: boolean) => void;
	}
</script>

<script lang="ts">
	import { tick, untrack, type Snippet } from 'svelte';
	import UserAvatar from '@carbon/icons/es/user--avatar/20.js';
	import { afterNavigate } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { setTheme, type Theme } from '#lib/theme.js';
	import Button from './Button.svelte';
	import { startsTrail } from '#lib/trail.svelte.js';
	import Icon from './Icon.svelte';
	import RadioButtonGroup from './RadioButtonGroup.svelte';
	import Toasts from './Toasts.svelte';

	interface Props {
		name: string;
		/** The product's logo, before its name. */
		logo?: string;
		nav: NavItem[];
		/** Who is logged in; null before logging in. */
		account: ShellAccount | null;
		/** The user's theme choice as rendered by the server. */
		theme: Theme;
		/** Further header actions, e.g. processing progress. */
		actions?: HeaderAction[];
		children: Snippet;
	}

	let { name, logo, nav, account, theme: initialTheme, actions = [], children }: Props = $props();

	let open = $state(false);
	let menuButton: HTMLButtonElement | undefined = $state();
	let navElement: HTMLElement | undefined = $state();

	// The profile panel: who is logged in, the theme and logging out. The theme
	// starts from the server's value, then follows the user's choices.
	// svelte-ignore state_referenced_locally
	let theme = $state(initialTheme);

	// The open header panel, by action id ('profile' or one of `actions`).
	// One panel at a time.
	let panelOpen: string | null = $state(null);
	const panelButtons: Record<string, HTMLButtonElement> = $state({});
	const panelElements: Record<string, HTMLElement> = $state({});

	// An action that disappears (e.g. the last controller got paired) closes its panel.
	$effect(() => {
		if (panelOpen && panelOpen !== 'profile' && !actions.some((a) => a.id === panelOpen)) {
			panelOpen = null;
		}
	});

	// Tells the actions when their panel opens or closes.
	let toggled: string | null = null;
	$effect(() => {
		const now = panelOpen;
		untrack(() => {
			if (now === toggled) return;
			actions.find((a) => a.id === toggled)?.ontoggle?.(false);
			actions.find((a) => a.id === now)?.ontoggle?.(true);
			toggled = now;
		});
	});

	afterNavigate(() => {
		open = false;
		panelOpen = null;
	});

	async function toggle() {
		open = !open;
		if (open) {
			panelOpen = null;
			// Move focus into the navigation, as Carbon's left panel does.
			await tick();
			navElement?.querySelector('a')?.focus();
		}
	}

	async function togglePanel(id: string) {
		panelOpen = panelOpen === id ? null : id;
		if (panelOpen) {
			open = false;
			await tick();
			const panel = panelElements[id];
			// Focus the checked option, as in Carbon's settings menus, or else the panel.
			(panel?.querySelector<HTMLInputElement>('input:checked') ?? panel)?.focus();
		}
	}

	function inPanel(node: Node | null) {
		if (!node || !panelOpen) return false;
		return !!(panelElements[panelOpen]?.contains(node) || panelButtons[panelOpen]?.contains(node));
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Escape') return;
		if (panelOpen) {
			panelButtons[panelOpen]?.focus();
			panelOpen = null;
		} else if (open) {
			open = false;
			menuButton?.focus();
		} else if (navElement?.contains(document.activeElement)) {
			// As Carbon's rail: leave it (which collapses it) for the content.
			document.getElementById('main-content')?.focus();
		}
	}

	/** Closes the panel on a click outside it or its button. */
	function onclick(event: MouseEvent) {
		// A target removed by the click itself (e.g. a re-rendered list) counts as inside.
		const target = event.target as Node;
		if (panelOpen && target.isConnected && !inPanel(target)) panelOpen = null;
	}

	/** Closes the panel when focus leaves it, e.g. by tabbing past the end. */
	function onfocusout(event: FocusEvent) {
		const next = event.relatedTarget as Node | null;
		if (panelOpen && next && !inPanel(next)) panelOpen = null;
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
	<a class="name" href={home} use:startsTrail>
		{#if logo}<img src={logo} alt="" width="20" height="20" />{/if}{name}
	</a>
	{#if account}
		<div class="utilities">
			{#each actions as action (action.id)}
				<button
					bind:this={panelButtons[action.id]}
					class="action"
					class:active={panelOpen === action.id}
					type="button"
					aria-label={action.label}
					aria-expanded={panelOpen === action.id}
					aria-controls="{action.id}-panel"
					onclick={() => togglePanel(action.id)}
					{onfocusout}
				>
					{@render action.icon()}
					<span class="tooltip" aria-hidden="true">{action.label}</span>
				</button>
				{#if panelOpen === action.id}
					<section
						bind:this={panelElements[action.id]}
						id="{action.id}-panel"
						class="panel layer-01"
						aria-label={action.label}
						tabindex="-1"
						{onfocusout}
					>
						{@render action.panel()}
					</section>
				{/if}
			{/each}
			<button
				bind:this={panelButtons.profile}
				class="action"
				class:active={panelOpen === 'profile'}
				type="button"
				aria-label="Profile"
				aria-expanded={panelOpen === 'profile'}
				aria-controls="profile-panel"
				onclick={() => togglePanel('profile')}
				{onfocusout}
			>
				<Icon icon={UserAvatar} />
				<span class="tooltip" aria-hidden="true">Profile</span>
			</button>
			{#if panelOpen === 'profile'}
				<section
					bind:this={panelElements.profile}
					id="profile-panel"
					class="panel layer-01"
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
	<nav
		bind:this={navElement}
		id="side-nav"
		class="theme-dark"
		class:open
		aria-label="Side navigation"
	>
		<ul>
			{#each nav as item (item.href)}
				<li>
					<!-- eslint-disable svelte/no-navigation-without-resolve -- resolved by the caller -->
					<a
						href={item.href}
						aria-current={isCurrent(item.href) ? 'page' : undefined}
						use:startsTrail
					>
						<!-- eslint-enable svelte/no-navigation-without-resolve -->
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

<Toasts />

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

	/* The logo 12px before the name, at the size of a header icon. */
	.name img {
		margin-right: var(--cds-spacing-04);
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

	/* Focused on opening, so keyboard users start inside it; not itself a control. */
	.panel:focus {
		outline: none;
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
		/* As wide as the rail while closed, so crossing the lg breakpoint
		   doesn't animate it from the menu's width. */
		width: var(--cds-size-large);
		padding-top: var(--cds-spacing-05);
		overflow: hidden auto;
		background: var(--cds-background);
		/* Hidden, not just moved, so closed links can't be focused. */
		visibility: hidden;
		transform: translateX(-100%);
		/* The width changes back only once it has slid out. */
		transition:
			transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9),
			visibility 110ms,
			width 0s 110ms;
	}

	/* Visible at once when opening, so focus can move into it; the delays
	   above only apply when closing. */
	nav.open {
		width: 16rem;
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

	/* Carbon's lg breakpoint (66rem): the rail, always visible, 48px wide. It
	   expands to 256px over the content on hover, after Carbon's 100ms delay so
	   passing over it doesn't flash the labels, or on keyboard focus (not
	   focus-within, so a clicked link doesn't keep it open), and collapses at
	   once. Custom properties can't be used in media queries. */
	@media (min-width: 66rem) {
		.menu,
		.scrim {
			display: none;
		}

		nav,
		nav.open {
			width: var(--cds-size-large);
			visibility: visible;
			transform: none;
			transition: width 110ms cubic-bezier(0.2, 0, 0.38, 0.9);
		}

		nav:hover,
		nav:has(:focus-visible) {
			width: 16rem;
			transition-delay: 100ms;
		}

		.name {
			padding-left: var(--cds-spacing-05);
		}

		main:not(.bare) {
			margin-left: var(--cds-size-large);
		}
	}
</style>
