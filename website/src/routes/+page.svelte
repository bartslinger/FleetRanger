<script lang="ts">
	import ArrowRight from '@carbon/icons/es/arrow--right/16.js';
	import Email from '@carbon/icons/es/email/16.js';
	import Launch from '@carbon/icons/es/launch/16.js';
	import Cloud from '@carbon/pictograms/es/cloud/index.js';
	import Servers from '@carbon/pictograms/es/servers/index.js';
	import Button from '$ui/Button.svelte';
	import Tag from '$ui/Tag.svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import Link from '$ui/Link.svelte';
	import Controller from '#lib/Controller.svelte';
	import Laptop from '#lib/Laptop.svelte';
	import mavic3 from '#lib/images/drone-mavic-3-enterprise.png';
	import matrice4 from '#lib/images/drone-matrice-4.png';
	import matrice30 from '#lib/images/drone-matrice-30.png';
	import matrice300 from '#lib/images/drone-matrice-300.png';
	import pilot640 from '#lib/images/pilot-640.webp';
	import pilot1280 from '#lib/images/pilot-1280.webp';
	import dark800 from '#lib/images/session-dark-800.webp';
	import dark1600 from '#lib/images/session-dark-1600.webp';
	import dark2400 from '#lib/images/session-dark-2400.webp';
	import light800 from '#lib/images/session-light-800.webp';
	import light1600 from '#lib/images/session-light-1600.webp';
	import light2400 from '#lib/images/session-light-2400.webp';
	import { APP, CONTACT, SOURCE } from '#lib/site.js';

	// The session page in the web app, in each theme.
	const session = {
		light: { src: light1600, srcset: `${light800} 800w, ${light1600} 1600w, ${light2400} 2400w` },
		dark: { src: dark1600, srcset: `${dark800} 800w, ${dark1600} 1600w, ${dark2400} 2400w` }
	};

	// Where it runs: on your own server first, the hosted service beside it.
	const points = [
		{
			pictogram: Servers,
			title: 'On your own premises',
			text: 'FleetRanger runs on your own servers, on site or in your own data centre. Your flight logs are stored there only.',
			link: { href: '/docs/self-hosting/requirements', text: 'Read the self-hosting docs' }
		},
		{
			pictogram: Cloud,
			title: 'Or hosted by us',
			text: `Rather not run a server? Use the hosted service at ${APP}.`,
			link: { href: '/pricing', text: 'See pricing' }
		}
	];

	const moments = [
		{
			when: 'On the controller',
			text: 'When the controller starts, FleetRanger asks who’s flying. One press on the pilot’s name opens DJI Pilot 2. After landing, the app uploads the flight logs by itself.',
			screen: {
				src: pilot1280,
				srcset: `${pilot640} 640w, ${pilot1280} 1280w`,
				alt: 'The FleetRanger app on a DJI RC Pro, asking who’s flying, with the last pilot first and all logs in FleetRanger.'
			}
		},
		{
			when: 'At your desk',
			text: 'Every flight is in the web app with its pilot, drone, times and track, and flights flown shortly after each other form a session.'
		}
	];

	// The drones whose logs FleetRanger reads: tested where a fleet flies
	// them, else expected to work, as they fly with DJI Pilot 2.
	const supported = [
		{ name: 'Mavic 3 Enterprise series', models: '3E, 3T, 3M', image: mavic3, tested: true },
		{ name: 'Matrice 4 series', models: '4E, 4T', image: matrice4, tested: false },
		{ name: 'Matrice 30 series', models: 'M30, M30T', image: matrice30, tested: false },
		{ name: 'Matrice 300, 350 and 400 RTK', models: '', image: matrice300, tested: false }
	];
	const features = [
		{
			title: 'Logbook and sessions',
			text: 'Every flight with its time in the air, distance, drone, controller and pilot. Sessions can be merged and split.'
		},
		{
			title: 'Fleet insights',
			text: 'Hours flown per drone, battery and group, and the areas where your fleet works.'
		},
		{
			title: 'Groups and equipment',
			text: 'Pilots work in groups, with roles per person and your own single sign-on. Drones and controllers are registered from their logs, so the fleet list keeps itself up to date.'
		},
		{
			title: 'Your own integrations',
			text: 'On your own server, your systems and AI assistants can connect through the API, with the same logins and permissions as your users. We can build the integration for you.'
		}
	];
</script>

<svelte:head>
	<title>FleetRanger: the flight logbook that fills itself</title>
	<meta
		name="description"
		content="Pilots pick their name on the controller, and every flight arrives with its pilot, drone, hours and track. Open-source flight logging for DJI Enterprise fleets, on your own servers."
	/>
</svelte:head>

<section class="hero" aria-labelledby="title">
	<Grid centered>
		<Column md={8} lg={7}>
			<div class="lead">
				<h1 id="title">The flight logbook that fills itself</h1>
				<div>
					<p class="intro">Open-source flight logging for DJI Enterprise, on your own server.</p>
					<div class="actions">
						<!-- The hosted free tier first (user, 2026-11-19), then self-hosting. -->
						<Button href="https://{APP}/signup" icon={ArrowRight}>Try for free</Button>
						<Button href="/docs/self-hosting/requirements" kind="tertiary" icon={ArrowRight}
							>Run it yourself</Button
						>
					</div>
				</div>
			</div>
		</Column>
		<Column md={8} lg={9}>
			<div class="hero-laptop">
				<Laptop
					{...session}
					width={56}
					alt="A session in FleetRanger: a flight's track on a satellite map, with its pilot, team, times and distance, and the list of its flights."
				/>
			</div>
		</Column>
	</Grid>
</section>

<section id="how-it-works" aria-labelledby="how-it-works-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column md={8} lg={4}>
			<h2 id="how-it-works-title">How it works</h2>
		</Column>
		<Column md={8} lg={12}>
			<ol class="steps">
				{#each moments as moment (moment.when)}
					<li>
						<div class="picture" class:laptop={!moment.screen}>
							{#if moment.screen}
								<Controller {...moment.screen} />
							{:else}
								<Laptop
									{...session}
									alt="A session in the web app: its flights' tracks on a map, with the pilot, drone, times and distance."
								/>
							{/if}
						</div>
						<h3>{moment.when}</h3>
						<p>{moment.text}</p>
					</li>
				{/each}
			</ol>
			<p class="body setup">
				A controller is set up once: install the app on it, and pair it with a code from the web
				app.
			</p>
			<p class="more">
				<Link href="/docs/user/getting-started" size="lg" icon={ArrowRight}>Getting started</Link>
			</p>
		</Column>
	</Grid>
</section>

<section id="features" aria-labelledby="features-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column md={8} lg={4}>
			<h2 id="features-title">Features</h2>
		</Column>
		<Column md={8} lg={12}>
			<ul class="tiles">
				{#each features as feature (feature.title)}
					<li>
						<h3>{feature.title}</h3>
						<p>{feature.text}</p>
					</li>
				{/each}
			</ul>
		</Column>
	</Grid>
</section>

<section aria-labelledby="points-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column md={8} lg={4}>
			<h2 id="points-title">Your data stays with you</h2>
		</Column>
		<Column md={8} lg={12}>
			<ul class="points">
				{#each points as point (point.title)}
					<li>
						<span class="pictogram"><Icon icon={point.pictogram} /></span>
						<h3>{point.title}</h3>
						<p>{point.text}</p>
						<p class="more">
							<Link href={point.link.href} size="lg" icon={ArrowRight}>{point.link.text}</Link>
						</p>
					</li>
				{/each}
			</ul>
		</Column>
	</Grid>
</section>

<section id="supported" aria-labelledby="supported-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column md={8} lg={4}>
			<h2 id="supported-title">Supported drones</h2>
		</Column>
		<Column md={8} lg={8}>
			<ul class="drones">
				{#each supported as drone (drone.name)}
					<li>
						<img src={drone.image} alt="" width="120" height="71" loading="lazy" />
						<span class="drone-name">
							{drone.name}{#if drone.models}<span class="models">{drone.models}</span>{/if}
						</span>
						<Tag type={drone.tested ? 'green' : 'gray'}
							>{drone.tested ? 'Tested' : 'Not tested yet'}</Tag
						>
					</li>
				{/each}
			</ul>
			<p class="body">
				The app runs on the DJI RC Pro Enterprise, RC Plus and RC Plus 2 Enterprise. Flying another
				DJI Enterprise drone with DJI Pilot 2? Write to {CONTACT}.
			</p>
		</Column>
	</Grid>
</section>

<section class="next" aria-labelledby="start-title">
	<Grid centered>
		<Column md={8} lg={8}>
			<h2 id="start-title">Open source, not open contribution</h2>
			<p class="body">
				FleetRanger is open source under the AGPL-3.0. You can run it, change it and keep using it,
				whatever happens to us. We don't accept pull requests, but ideas and feature requests are
				welcome at {CONTACT}. When an organisation funds a feature, it comes first. It ships in
				FleetRanger itself, so everyone gets it.
			</p>
			<div class="actions">
				<Button href="mailto:{CONTACT}" icon={Email}>Get in touch</Button>
				<Button href={SOURCE} kind="tertiary" icon={Launch}>View source code</Button>
			</div>
		</Column>
	</Grid>
</section>

<style>
	section {
		padding-block: var(--cds-spacing-09);
	}

	@media (min-width: 66rem) {
		section {
			padding-block: var(--cds-spacing-10);
		}
	}

	section[id] {
		scroll-margin-top: var(--cds-spacing-05);
	}

	/* Content sections start at a thin rule above their heading, as on IBM's
	   pages, as wide as the grid's content. The heading follows a gutter
	   (spacing-07) below it: the grid's row gap, or this margin without one. */
	.rule {
		border-top: 1px solid var(--cds-border-subtle-00);
	}

	/* The headline: Carbon's largest expressive heading, semibold. */
	h1 {
		font-size: var(--cds-fluid-heading-06-font-size);
		font-weight: var(--cds-fluid-heading-06-font-weight);
		line-height: var(--cds-fluid-heading-06-line-height);
		letter-spacing: var(--cds-fluid-heading-06-letter-spacing);
	}

	h2 {
		font-size: var(--cds-fluid-heading-04-font-size);
		font-weight: var(--cds-fluid-heading-04-font-weight);
		line-height: var(--cds-fluid-heading-04-line-height);
		letter-spacing: var(--cds-fluid-heading-04-letter-spacing);
	}

	h3 {
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	p {
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
		letter-spacing: var(--cds-body-02-letter-spacing);
	}

	.intro {
		max-width: 36rem;
		color: var(--cds-text-secondary);
	}

	.body {
		max-width: 36rem;
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-08);
	}

	/* The hero: the text beside the laptop from lg, above it before that. */
	.lead {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
		height: 100%;
	}

	.hero-laptop {
		max-width: 40rem;
		margin-top: var(--cds-spacing-09);
	}

	@media (min-width: 66rem) {
		.lead {
			justify-content: center;
			min-height: 26rem;
		}

		/* As wide as its nine columns. */
		.hero-laptop {
			display: flex;
			align-items: center;
			max-width: none;
			height: 100%;
			margin: 0;
			padding-inline-start: var(--cds-spacing-07);
		}

		.hero-laptop > :global(*) {
			flex: 1;
		}
	}

	/* Points with Carbon pictograms, as on IBM's product pages, in the
	   features' columns from lg. */
	.points {
		display: grid;
		gap: var(--cds-spacing-09) var(--cds-spacing-07);
	}

	@media (min-width: 42rem) {
		.points {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	.pictogram {
		display: block;
		margin-bottom: var(--cds-spacing-07);
		color: var(--cds-icon-interactive);
	}

	.pictogram :global(svg) {
		width: 4rem;
		height: 4rem;
	}

	.points h3 {
		font-size: var(--cds-heading-02-font-size);
		font-weight: var(--cds-heading-02-font-weight);
		line-height: var(--cds-heading-02-line-height);
	}

	.points p {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}

	/* Carbon's standalone link, below the text it belongs to. */
	.more,
	.points p.more {
		margin-top: var(--cds-spacing-05);
	}

	/* Two steps side by side from md, each with its picture on top. */
	.steps {
		display: grid;
		gap: var(--cds-spacing-09) var(--cds-spacing-07);
	}

	@media (min-width: 42rem) {
		.steps {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	/* One height for both pictures, bottom-aligned, so the titles line up: the
	   controller is as tall as it is 4/3 wide, the laptop, a little wider than a
	   controller in life, 5/3. Small on phones, so the section stays in
	   proportion with the rest. */
	.picture {
		display: flex;
		align-items: flex-end;
		height: 7.5rem;
		max-width: 10rem;
		margin-bottom: var(--cds-spacing-06);
	}

	.picture.laptop {
		max-width: 12.5rem;
	}

	@media (min-width: 42rem) {
		.picture {
			height: 12rem;
			max-width: 16rem;
		}

		.picture.laptop {
			max-width: 20rem;
		}
	}

	.picture > :global(*) {
		width: 100%;
	}

	.steps p {
		max-width: 26rem;
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}

	/* Setting up, once, below the steps: a sentence and the docs link. */
	.setup {
		margin-top: var(--cds-spacing-09);
	}

	/* Carbon tiles on the page, with the narrow grid's 16px gutter. */
	.tiles {
		display: grid;
		gap: var(--cds-spacing-05);
	}

	@media (min-width: 42rem) {
		.tiles {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	.tiles li {
		padding: var(--cds-spacing-05) var(--cds-spacing-05) var(--cds-spacing-07);
		background: var(--cds-layer-01);
	}

	.tiles p {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}

	/* The supported drones: not Carbon's structured list, built for text
	   columns, as the photos are the point (carbon-expert's advice); a picture,
	   the name with its
	   models under it, and a tag for whether it is tested; Carbon's 1px rule
	   between rows. */
	.drones li {
		display: flex;
		align-items: center;
		gap: var(--cds-spacing-05);
		padding-block: var(--cds-spacing-03);
		border-bottom: 1px solid var(--cds-border-subtle-01);
	}

	.drones img {
		flex: none;
		width: 6rem;
		/* Product photos, most on white: all on white, also in the dark theme. */
		background: #ffffff;
		height: auto;
	}

	.drone-name {
		flex: 1;
	}

	.models {
		display: block;
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	#supported .body {
		margin-top: var(--cds-spacing-06);
	}

	/* The next step: a Gray 10 band before the footer. */
	.next {
		background: var(--cds-layer-01);
	}

	.next .body {
		margin-top: var(--cds-spacing-06);
	}
</style>
