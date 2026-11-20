<script lang="ts">
	import ArrowRight from '@carbon/icons/es/arrow--right/16.js';
	import Email from '@carbon/icons/es/email/16.js';
	import Button from '$ui/Button.svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Tag from '$ui/Tag.svelte';
	import { APP, CONTACT } from '#lib/site.js';
</script>

<svelte:head>
	<title>Pricing · FleetRanger</title>
	<meta
		name="description"
		content="FleetRanger is free to run on your own server. It is also available as a hosted service, free for small teams."
	/>
</svelte:head>

<section aria-labelledby="title">
	<Grid centered rowGap>
		<Column md={8} lg={8}>
			<h1 id="title">Pricing</h1>
			<p class="intro">
				FleetRanger is made to run on your own server, so your flight logs stay with you. Rather not
				run a server? Use it as a hosted service at {APP}.
			</p>
		</Column>
		<Column>
			<!-- Carbon's tiles, one per option, the recommended one first and the
			     only primary button on the page (carbon-expert). -->
			<ul class="options">
				<li>
					<Tag type="blue">Recommended</Tag>
					<h2>Self-hosted</h2>
					<p class="price">Free</p>
					<ul class="included">
						<li>On your own server</li>
						<li>No limits on users or controllers</li>
						<li>Integrations through the API</li>
						<li>Open source, AGPL-3.0</li>
						<li>Installation and support on request, paid</li>
					</ul>
					<div class="action">
						<Button href="/docs/self-hosting/requirements" icon={ArrowRight} fill
							>Read the docs</Button
						>
					</div>
				</li>
				<li>
					<h2>Hosted</h2>
					<p class="price">Free</p>
					<ul class="included">
						<li>Hosted at {APP}</li>
						<li>2 users who log in</li>
						<li>2 controllers</li>
						<li>Any number of pilots who only fly</li>
					</ul>
					<div class="action">
						<Button href="https://{APP}/signup" kind="tertiary" icon={ArrowRight} fill
							>Try for free</Button
						>
					</div>
				</li>
				<li>
					<h2>Hosted, no limits</h2>
					<p class="price">Contact us</p>
					<ul class="included">
						<li>Hosted at {APP}</li>
						<li>No limits on users or controllers</li>
					</ul>
					<div class="action">
						<Button href="mailto:{CONTACT}" kind="tertiary" icon={Email} fill>Get in touch</Button>
					</div>
				</li>
			</ul>
		</Column>
		<Column md={4} lg={8}>
			<div class="about">
				<h2>Open source, not open contribution</h2>
				<p>
					FleetRanger is open source under the AGPL-3.0. You can run it, change it and keep using
					it, whatever happens to us. We don't accept pull requests, but ideas and feature requests
					are welcome at <a href="mailto:{CONTACT}">{CONTACT}</a>. When an organisation funds a
					feature, it comes first. It ships in FleetRanger itself, so everyone gets it.
				</p>
			</div>
		</Column>
		<Column md={4} lg={8}>
			<div class="about">
				<h2>Your own integrations</h2>
				<p>
					On your own server, your systems can connect to FleetRanger through its API, with the same
					logins and permissions as your users. We can build the integration for you.
				</p>
			</div>
		</Column>
	</Grid>
</section>

<style>
	section {
		padding-block: var(--cds-spacing-09) var(--cds-spacing-10);
	}

	h1 {
		font-size: var(--cds-fluid-heading-05-font-size);
		font-weight: var(--cds-fluid-heading-05-font-weight);
		line-height: var(--cds-fluid-heading-05-line-height);
		letter-spacing: var(--cds-fluid-heading-05-letter-spacing);
	}

	.intro {
		max-width: 36rem;
		margin-top: var(--cds-spacing-06);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
		letter-spacing: var(--cds-body-02-letter-spacing);
	}

	/* Stacked on phones and tablets, three of equal height from lg, with the
	   narrow grid's 16px gutter as the home page's tiles. */
	.options {
		display: grid;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-07);
	}

	@media (min-width: 66rem) {
		.options {
			grid-template-columns: repeat(3, minmax(0, 1fr));
		}
	}

	/* The button at the bottom, so the three line up. */
	.options > li {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
	}

	/* The tag above the name. Side by side, the others leave its room empty,
	   so the names line up too. */
	.options > li > :global(.tag) {
		margin-bottom: var(--cds-spacing-03);
	}

	@media (min-width: 66rem) {
		.options > li:not(:first-child) h2 {
			margin-top: calc(var(--cds-spacing-06) + var(--cds-spacing-03));
		}
	}

	h2 {
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	.price {
		margin-top: var(--cds-spacing-05);
		font-size: var(--cds-heading-05-font-size);
		font-weight: var(--cds-heading-05-font-weight);
		line-height: var(--cds-heading-05-line-height);
		letter-spacing: var(--cds-heading-05-letter-spacing);
	}

	/* Carbon's unordered list: an en dash before each item, in the expressive
	   body size. */
	.included {
		margin-top: var(--cds-spacing-06);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
		letter-spacing: var(--cds-body-02-letter-spacing);
	}

	.included li {
		position: relative;
		padding-inline-start: var(--cds-spacing-05);
	}

	.included li::before {
		position: absolute;
		inset-inline-start: 0;
		content: '–';
	}

	/* What the options share, below them: plain text with a gap above. */
	.about {
		max-width: 36rem;
		margin-top: var(--cds-spacing-09);
	}

	.about p {
		margin-top: var(--cds-spacing-04);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
		letter-spacing: var(--cds-body-02-letter-spacing);
	}

	/* The buttons span the tile, as Carbon has them in base tiles. */
	.action {
		align-self: stretch;
		margin-top: auto;
		padding-top: var(--cds-spacing-08);
	}
</style>
