<script lang="ts">
	import '$ui/base.css';
	import Asleep from '@carbon/icons/es/asleep/20.js';
	import Launch from '@carbon/icons/es/launch/16.js';
	import Light from '@carbon/icons/es/light/20.js';
	import logo from '$assets/favicon.svg';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import { CONTACT, SOURCE } from '#lib/site.js';

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
	<nav aria-label="Sections">
		<a href="/#privacy">Privacy</a>
		<a href="/#how-it-works">How it works</a>
		<a href="/#features">Features</a>
	</nav>
	<!-- Carbon's header actions; the one for the other theme shows (app.html). -->
	<div class="actions">
		<button type="button" class="to-light" data-set-theme="light" aria-label="Light theme">
			<Icon icon={Light} />
		</button>
		<button type="button" class="to-dark" data-set-theme="dark" aria-label="Dark theme">
			<Icon icon={Asleep} />
		</button>
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

	/* Tighter on phones so the links fit beside the name, and left out where
	   they don't: the sections follow each other on the one page. */
	nav {
		display: none;
	}

	@media (min-width: 30rem) {
		nav {
			display: flex;
		}
	}

	nav a {
		padding-inline: var(--cds-spacing-03);
		white-space: nowrap;
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

	/* Shown only with the script that makes them work. */
	.actions button {
		display: none;
		align-items: center;
		justify-content: center;
		width: var(--cds-size-large);
		height: var(--cds-size-large);
		padding: 0;
		border: 0;
		background: none;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	.actions button:hover {
		background: var(--cds-background-hover);
	}

	.actions button:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	:global(.js) .to-dark {
		display: flex;
	}

	@media (prefers-color-scheme: dark) {
		:global(.js:not([data-theme='light'])) .to-dark {
			display: none;
		}

		:global(.js:not([data-theme='light'])) .to-light {
			display: flex;
		}
	}

	:global(.js[data-theme='dark']) .to-dark {
		display: none;
	}

	:global(.js[data-theme='dark']) .to-light {
		display: flex;
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

	footer a:hover {
		color: var(--cds-link-primary-hover);
		text-decoration: underline;
	}
</style>
