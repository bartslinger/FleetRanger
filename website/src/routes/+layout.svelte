<script lang="ts">
	import '$ui/base.css';
	import { page } from '$app/state';
	import Asleep from '@carbon/icons/es/asleep/20.js';
	import Launch from '@carbon/icons/es/launch/16.js';
	import Light from '@carbon/icons/es/light/20.js';
	import Screen from '@carbon/icons/es/screen/20.js';
	import logo from '$assets/favicon.svg';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import RadioButtonGroup from '$ui/RadioButtonGroup.svelte';
	import { APP, CONTACT, SOURCE } from '#lib/site.js';

	let { children } = $props();
</script>

<svelte:head>
	<meta name="color-scheme" content="light dark" />
	<meta name="theme-color" content="#161616" />
	<link rel="icon" href={logo} />
</svelte:head>

<a class="skip" href="#main">Skip to main content</a>

<header class="theme-dark">
	<a class="name" href="/"><img src={logo} alt="" width="20" height="20" />FleetRanger</a>
	<nav aria-label="Main">
		<a
			class="pricing"
			href="/pricing"
			aria-current={page.url.pathname.startsWith('/pricing') ? 'true' : undefined}>Pricing</a
		>
		<a href="/docs" aria-current={page.url.pathname.startsWith('/docs') ? 'true' : undefined}
			>Docs</a
		>
	</nav>
	<div class="actions">
		<!-- For those who already use the hosted service: a text link, as
		     visitors would miss an icon. Before the theme action, which keeps
		     the far right, where Carbon puts icon actions. -->
		<a class="login" href="https://{APP}/">Log in</a>
		<!-- Carbon's header action with a panel, as the app's profile panel: the
		     theme choice, with the chosen one's icon on the button. The browser's
		     popover opens and closes it; the script in app.html applies the choice. -->
		<button type="button" class="action" popovertarget="theme-panel" aria-label="Theme">
			<span class="system"><Icon icon={Screen} /></span>
			<span class="light"><Icon icon={Light} /></span>
			<span class="dark"><Icon icon={Asleep} /></span>
		</button>
		<section id="theme-panel" class="panel layer-01" popover aria-label="Theme">
			<RadioButtonGroup
				legend="Theme"
				name="theme"
				options={[
					{ value: 'system', label: 'System' },
					{ value: 'light', label: 'Light' },
					{ value: 'dark', label: 'Dark' }
				]}
				value="system"
			/>
		</section>
	</div>
</header>

<main id="main">
	{@render children()}
</main>

<footer class="theme-dark">
	<Grid centered rowGap>
		<Column md={4} lg={4}>
			<p class="name"><img src={logo} alt="" width="24" height="24" />FleetRanger</p>
		</Column>
		<Column md={4} lg={4}>
			<p class="label">Source code</p>
			<a href={SOURCE}>FleetRanger repository <Icon icon={Launch} /></a>
		</Column>
		<Column md={4} lg={4}>
			<p class="label">License</p>
			<a href="https://www.gnu.org/licenses/agpl-3.0.html">AGPL-3.0 <Icon icon={Launch} /></a>
		</Column>
		<Column md={4} lg={4}>
			<p class="label">Contact</p>
			<a href="mailto:{CONTACT}">{CONTACT}</a>
		</Column>
		<Column md={4} lg={4}>
			<p class="label">Hosted service</p>
			<p class="links">
				<a href="/terms">Terms of use</a>
				<a href="/privacy">Privacy policy</a>
				<a href="/processing-agreement">Processing agreement</a>
			</p>
		</Column>
	</Grid>
</footer>

<style>
	.skip {
		position: absolute;
		inset-inline-start: var(--cds-spacing-05);
		top: var(--cds-spacing-03);
		z-index: 1;
		padding: var(--cds-spacing-03) var(--cds-spacing-05);
		background: var(--cds-background);
		transform: translateY(-200%);
	}

	.skip:focus {
		transform: none;
	}

	/* Carbon's header: 48px, the dark shell colours. */
	header {
		display: flex;
		align-items: stretch;
		min-height: var(--cds-size-large);
		border-bottom: 1px solid var(--cds-border-subtle-01);
	}

	header a {
		display: flex;
		align-items: center;
		padding-inline: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
	}

	header a:hover {
		background: var(--cds-background-hover);
		color: var(--cds-text-primary);
	}

	header .name {
		color: var(--cds-text-primary);
		font-weight: 600;
	}

	/* The logo 12px before the name, as in the app's header. */
	.name img {
		margin-right: var(--cds-spacing-04);
	}

	footer .name {
		display: flex;
		align-items: center;
	}

	/* Tighter on phones so the links fit beside the name. Only real pages:
	   the landing page's sections follow each other anyway. */
	nav {
		display: flex;
	}

	/* Pricing from 360px, where it fits beside Docs and Log in. */
	nav a.pricing {
		display: none;
	}

	@media (min-width: 22.5rem) {
		nav a.pricing {
			display: flex;
		}
	}

	nav a {
		padding-inline: var(--cds-spacing-03);
		white-space: nowrap;
	}

	/* The section the page is in: Carbon's selected header link. */
	nav a[aria-current] {
		box-shadow: inset 0 -2px 0 var(--cds-border-interactive);
		color: var(--cds-text-primary);
	}

	@media (min-width: 42rem) {
		nav a {
			padding-inline: var(--cds-spacing-05);
		}
	}

	.actions {
		display: flex;
		margin-inline-start: auto;
	}

	/* Tighter on phones, as the nav links, so it stays beside them. */
	.login {
		padding-inline: var(--cds-spacing-03);
		white-space: nowrap;
	}

	@media (min-width: 42rem) {
		.login {
			padding-inline: var(--cds-spacing-05);
		}
	}

	/* As the app's header actions. Shown only with the script that makes the
	   choice work. */
	.action {
		display: none;
		align-items: center;
		justify-content: center;
		width: var(--cds-size-large);
		height: var(--cds-size-large);
		padding: 0;
		border: 1px solid transparent;
		background: none;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	:global(.js) .action {
		display: flex;
	}

	.action:hover {
		background: var(--cds-background-hover);
	}

	.action:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Open: outlined, with the bottom edge flowing into the panel. */
	.actions:has(.panel:popover-open) .action {
		border-color: var(--cds-border-subtle-00);
		border-bottom-color: transparent;
		background: var(--cds-layer-01);
	}

	/* The icon of the choice: system unless the visitor chose a theme. */
	.action span {
		display: none;
	}

	:global(:root:not([data-theme])) .system,
	:global(:root[data-theme='light']) .light,
	:global(:root[data-theme='dark']) .dark {
		display: flex;
	}

	/* The app's header panel, below the header at the right, only as tall as
	   its contents. Undoes the browser's popover styles. */
	.panel {
		inset: var(--cds-size-large) 0 auto auto;
		width: 16rem;
		margin: 0;
		padding: var(--cds-spacing-05);
		border: 1px solid var(--cds-border-subtle-00);
		border-top: 0;
		background: var(--cds-layer-01);
		color: var(--cds-text-primary);
	}

	/* The website's one colour outside Carbon's tokens: the app's track colour
	   (Carbon magenta 50, as in TrackMap.svelte), the same in both themes. */
	:global(:root) {
		--track: #ee5396;
	}

	footer {
		padding-block: var(--cds-spacing-10);
	}

	/* On a dark page the footer has the page's colour: a line sets it apart. */
	@media (prefers-color-scheme: dark) {
		:global(:root:not([data-theme='light'])) footer {
			border-top: 1px solid var(--cds-border-subtle-01);
		}
	}

	:global(:root[data-theme='dark']) footer {
		border-top: 1px solid var(--cds-border-subtle-01);
	}

	footer .name {
		font-size: var(--cds-heading-02-font-size);
		font-weight: var(--cds-heading-02-font-weight);
		line-height: var(--cds-heading-02-line-height);
	}

	footer .label {
		margin-bottom: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	/* Carbon's standalone link, with the launch icon for other sites. */
	footer a {
		display: inline-flex;
		align-items: center;
		gap: var(--cds-spacing-02);
		color: var(--cds-link-primary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		text-decoration: none;
	}

	/* Several links under one label, below each other. */
	footer .links {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--cds-spacing-03);
	}

	footer a:hover {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}
</style>
