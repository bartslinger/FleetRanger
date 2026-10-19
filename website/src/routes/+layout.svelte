<script lang="ts">
	import '$ui/base.css';
	import Launch from '@carbon/icons/es/launch/16.js';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import { CONTACT, SOURCE } from '#lib/site.js';

	let { children } = $props();
</script>

<svelte:head>
	<meta name="color-scheme" content="light dark" />
	<meta name="theme-color" content="#161616" />
</svelte:head>

<a class="skip" href="#main">Skip to main content</a>

<header class="theme-dark">
	<a class="name" href="/">FleetRanger</a>
	<nav aria-label="Sections">
		<a href="/#how-it-works">How it works</a>
		<a href="/#features">Features</a>
		<a href="/#privacy">Privacy</a>
	</nav>
</header>

<main id="main">
	{@render children()}
</main>

<footer class="theme-dark">
	<Grid centered rowGap>
		<Column md={4} lg={4}>
			<p class="name">FleetRanger</p>
		</Column>
		<Column md={4} lg={4}>
			<h2>Source code</h2>
			<a href={SOURCE}>Repository <Icon icon={Launch} /></a>
		</Column>
		<Column md={4} lg={4}>
			<h2>License</h2>
			<a href="https://www.gnu.org/licenses/agpl-3.0.html">AGPL-3.0 <Icon icon={Launch} /></a>
		</Column>
		<Column md={4} lg={4}>
			<h2>Contact</h2>
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

	nav {
		display: none;
	}

	@media (min-width: 42rem) {
		nav {
			display: flex;
		}
	}

	footer {
		padding-block: var(--cds-spacing-10);
	}

	/* On a dark page the footer has the page's colour: a line sets it apart. */
	@media (prefers-color-scheme: dark) {
		footer {
			border-top: 1px solid var(--cds-border-subtle-01);
		}
	}

	footer .name {
		font-size: var(--cds-heading-02-font-size);
		font-weight: var(--cds-heading-02-font-weight);
		line-height: var(--cds-heading-02-line-height);
	}

	footer h2 {
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
