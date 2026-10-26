<script lang="ts">
	import Email from '@carbon/icons/es/email/16.js';
	import Launch from '@carbon/icons/es/launch/16.js';
	import Contract from '@carbon/pictograms/es/contract/index.js';
	import Drone from '@carbon/pictograms/es/drone/index.js';
	import Insights from '@carbon/pictograms/es/insights/index.js';
	import Keystores from '@carbon/pictograms/es/keystores/index.js';
	import ListCheckbox from '@carbon/pictograms/es/list--checkbox/index.js';
	import OpenSource from '@carbon/pictograms/es/open-source/index.js';
	import Servers from '@carbon/pictograms/es/servers/index.js';
	import Button from '$ui/Button.svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import Controller from '#lib/Controller.svelte';
	import Laptop from '#lib/Laptop.svelte';
	import pilot640 from '#lib/images/pilot-640.webp';
	import pilot1280 from '#lib/images/pilot-1280.webp';
	import dark800 from '#lib/images/session-dark-800.webp';
	import dark1600 from '#lib/images/session-dark-1600.webp';
	import dark2400 from '#lib/images/session-dark-2400.webp';
	import light800 from '#lib/images/session-light-800.webp';
	import light1600 from '#lib/images/session-light-1600.webp';
	import light2400 from '#lib/images/session-light-2400.webp';
	import sync640 from '#lib/images/sync-640.webp';
	import sync1280 from '#lib/images/sync-1280.webp';
	import { CONTACT, SOURCE } from '#lib/site.js';

	// The session page in the web app, in each theme.
	const session = {
		light: { src: light1600, srcset: `${light800} 800w, ${light1600} 1600w, ${light2400} 2400w` },
		dark: { src: dark1600, srcset: `${dark800} 800w, ${dark1600} 1600w, ${dark2400} 2400w` }
	};

	const points = [
		{
			pictogram: Servers,
			title: 'On your own premises',
			text: 'FleetRanger runs on your own servers, on site or in your own data centre. Your flight logs are stored there only.'
		},
		{
			pictogram: Keystores,
			title: 'Permissions for every team',
			text: 'Roles per group or for the whole organisation, with your own single sign-on.'
		},
		{
			pictogram: OpenSource,
			title: 'Open source',
			text: 'Licensed under the AGPL-3.0, so you can check what it does with your data.'
		},
		{
			pictogram: Drone,
			title: 'Made for DJI Enterprise',
			text: 'For drones flown with DJI Pilot 2, on controllers such as the DJI RC Pro.'
		}
	];

	const pains = [
		{
			pictogram: Contract,
			title: 'No more logging by hand',
			text: 'Pilots just fly. Every flight is recorded without them having to write anything down.'
		},
		{
			pictogram: ListCheckbox,
			title: 'Every flight accounted for',
			text: 'Pilots pick their name on the controller, so every flight lands in the logbook with its pilot, drone and track.'
		},
		{
			pictogram: Insights,
			title: 'Fleet insights',
			text: 'Hours flown per drone, battery and group, and the areas where your fleet works.'
		}
	];

	const moments = [
		{
			when: 'Before take-off',
			text: 'The pilot picks their name on the controller, even without a connection, and FleetRanger opens DJI Pilot 2.',
			screen: {
				src: pilot1280,
				srcset: `${pilot640} 640w, ${pilot1280} 1280w`,
				alt: 'The FleetRanger app on a DJI RC Pro, asking who’s flying, with the last pilot first.'
			}
		},
		{
			when: 'After landing',
			text: 'The controller uploads the new flight logs to your server, over Wi-Fi networks you approve.',
			screen: {
				src: sync1280,
				srcset: `${sync640} 640w, ${sync1280} 1280w`,
				alt: 'The app’s sync settings on a DJI RC Pro: all logs uploaded.'
			}
		},
		{
			when: 'At your desk',
			text: 'Flights that follow each other are grouped into sessions, with their times, pilots, drones and tracks on a map.'
		}
	];

	const features = [
		{
			title: 'An app on the controller',
			text: 'Pilots choose their name before each flight. Nobody logs in on a shared controller.'
		},
		{
			title: 'Logbook and sessions',
			text: 'Every flight with its time in the air, distance, drone, controller and pilot. Sessions can be merged and split.'
		},
		{
			title: 'Connect your own apps',
			text: 'Apps and AI assistants sign in through OAuth and see only what their user may see. Admins add trusted apps and give their roles per person.'
		},
		{
			title: 'Groups and equipment',
			text: 'Pilots work in groups, with roles per person. Drones and controllers are registered from their logs, so the fleet list keeps itself up to date.'
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
						<Button href="mailto:{CONTACT}" icon={Email}>Get in touch</Button>
						<Button href={SOURCE} kind="tertiary" icon={Launch}>View source code</Button>
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

<section aria-labelledby="pains-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column>
			<h2 id="pains-title">Less paperwork, more insight</h2>
		</Column>
		<Column>
			<ul class="pains">
				{#each pains as pain (pain.title)}
					<li>
						<span class="pictogram"><Icon icon={pain.pictogram} /></span>
						<h3>{pain.title}</h3>
						<p>{pain.text}</p>
					</li>
				{/each}
			</ul>
		</Column>
	</Grid>
</section>

<section aria-labelledby="points-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column>
			<h2 id="points-title">Your data stays with you</h2>
		</Column>
		<Column>
			<ul class="points">
				{#each points as point (point.title)}
					<li>
						<span class="pictogram"><Icon icon={point.pictogram} /></span>
						<h3>{point.title}</h3>
						<p>{point.text}</p>
					</li>
				{/each}
			</ul>
		</Column>
	</Grid>
</section>

<section id="how-it-works" aria-labelledby="how-it-works-title">
	<Grid centered>
		<Column><div class="rule"></div></Column>
		<Column>
			<h2 id="how-it-works-title">How it works</h2>
		</Column>
		<Column>
			<!-- The steps scroll past the picture, which stays in place and shows the
			     step that has reached it (the script in app.html). -->
			<div class="steps" data-steps>
				<div class="picture" data-picture>
					{#each moments as moment, i (moment.when)}
						<div class="shown" class:active={i === 0} data-shown={i}>
							{#if moment.screen}
								<Controller {...moment.screen} />
							{:else}
								<Laptop {...session} alt="The session in the web app on a laptop." />
							{/if}
						</div>
					{/each}
				</div>
				<ol class="timeline">
					{#each moments as moment, i (moment.when)}
						<li class:last={i === moments.length - 1} data-step={i}>
							<h3>{moment.when}</h3>
							<p>{moment.text}</p>
						</li>
					{/each}
				</ol>
			</div>
		</Column>
	</Grid>
</section>

<section class="quote" aria-label="What a pilot says">
	<Grid centered>
		<Column md={8} lg={8}>
			<figure>
				<blockquote>
					<p>
						Our flight logs are too sensitive for the cloud. FleetRanger keeps them on our own
						server, and makes our operations much simpler.
					</p>
				</blockquote>
				<figcaption>Drone pilot and satisfied FleetRanger user</figcaption>
			</figure>
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

<section class="next" aria-labelledby="start-title">
	<Grid centered>
		<Column md={8} lg={8}>
			<h2 id="start-title">Set it up for your fleet</h2>
			<p class="body">
				FleetRanger is free to run yourself. Need a feature for your organisation? We build it into
				FleetRanger, so you keep getting new versions. Write to {CONTACT}.
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

	#how-it-works .rule {
		margin-bottom: var(--cds-spacing-07);
	}

	/* The quote breaks the rhythm: more space around it, and no rule. */
	.quote {
		padding-block: var(--cds-spacing-11);
	}

	@media (min-width: 66rem) {
		.quote {
			padding-block: var(--cds-spacing-12);
		}
	}

	/* A full gap before the closing band, whose Gray 10 the tiles share. */
	#features {
		padding-bottom: var(--cds-spacing-10);
	}

	@media (min-width: 66rem) {
		#features {
			padding-bottom: var(--cds-spacing-11);
		}
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

	/* Four points with Carbon pictograms, as on IBM's product pages. */
	.points {
		display: grid;
		gap: var(--cds-spacing-09) var(--cds-spacing-07);
		margin-top: var(--cds-spacing-09);
	}

	@media (min-width: 42rem) {
		.points {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	@media (min-width: 66rem) {
		.points {
			grid-template-columns: repeat(4, minmax(0, 1fr));
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

	/* A pilot's words in Carbon's quotation style, the opening
	   quotation mark hanging in the margin so the text lines up. */
	.quote blockquote p {
		position: relative;
		font-size: var(--cds-fluid-quotation-01-font-size);
		font-weight: var(--cds-fluid-quotation-01-font-weight);
		line-height: var(--cds-fluid-quotation-01-line-height);
		letter-spacing: var(--cds-fluid-quotation-01-letter-spacing);
	}

	.quote blockquote p::before {
		content: '“';
	}

	/* On phones the margin is too narrow to hang it in. */
	@media (min-width: 42rem) {
		.quote blockquote p::before {
			position: absolute;
			right: 100%;
			padding-inline-end: 0.1em;
		}
	}

	.quote blockquote p::after {
		content: '”';
	}

	.quote figcaption {
		margin-top: var(--cds-spacing-07);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
	}

	/* The picture stays in place while the steps scroll past it: at the top on
	   phones, beside them from md, where it stops in the middle of the window
	   (it is 24rem tall at most). */
	.steps {
		display: grid;
		margin-top: var(--cds-spacing-07);
	}

	.picture {
		position: sticky;
		top: 0;
		z-index: 1;
		display: grid;
		align-items: center;
		padding-block: var(--cds-spacing-05);
		background: var(--cds-background);
	}

	/* The pictures on top of each other; the one for the current step shows. */
	.shown {
		grid-area: 1 / 1;
		max-width: 20rem;
		opacity: 0;
		transition: opacity 400ms cubic-bezier(0.2, 0, 0.38, 0.9);
	}

	.shown.active {
		opacity: 1;
	}

	@media (prefers-reduced-motion: reduce) {
		.shown {
			transition: none;
		}
	}

	@media (min-width: 42rem) {
		.steps {
			grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
			gap: var(--cds-spacing-07);
		}

		.picture {
			grid-column: 2;
			grid-row: 1;
			align-self: start;
			top: max(var(--cds-spacing-09), calc(50vh - 12rem));
			padding-block: 0;
			background: none;
		}

		.shown {
			max-width: 32rem;
		}
	}

	/* Three moments along a line, marked like a track: a ring at take-off,
	   dots after it. Each is tall enough to be read while its picture shows;
	   the last one must pass the switching line at 40% of the window before
	   the picture's bottom (12rem below the middle at most) moves on: 10vh +
	   12rem, and some room. */
	.timeline {
		display: grid;
		gap: var(--cds-spacing-05);
	}

	.timeline li {
		position: relative;
		min-height: 30vh;
		padding-inline-start: var(--cds-spacing-08);
	}

	@media (min-width: 42rem) {
		.timeline {
			grid-column: 1;
			grid-row: 1;
		}

		.timeline li {
			min-height: 16rem;
		}

		.timeline li.last {
			min-height: calc(10vh + 14rem);
		}
	}

	.timeline li::before {
		content: '';
		position: absolute;
		top: 0.375rem;
		left: 0;
		width: 0.875rem;
		height: 0.875rem;
		border: 3px solid var(--track);
		border-radius: 50%;
		background: var(--track);
		box-sizing: border-box;
	}

	.timeline li:first-child::before {
		background: var(--cds-background);
	}

	.timeline li::after {
		content: '';
		position: absolute;
		top: 1.5rem;
		bottom: calc(-1 * var(--cds-spacing-05) + 0.25rem);
		left: calc(0.4375rem - 1px);
		border-left: 2px solid var(--cds-border-subtle-01);
	}

	.timeline li.last::after {
		display: none;
	}

	.timeline p {
		max-width: 26rem;
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
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

	/* The pains as base tiles with a pictogram, unlike the points' open list. */
	.pains {
		display: grid;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-09);
	}

	@media (min-width: 66rem) {
		.pains {
			grid-template-columns: repeat(3, minmax(0, 1fr));
		}
	}

	.pains li {
		padding: var(--cds-spacing-05) var(--cds-spacing-05) var(--cds-spacing-07);
		background: var(--cds-layer-01);
	}

	.pains p {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}

	/* The next step: a Gray 10 band before the footer. */
	.next {
		background: var(--cds-layer-01);
	}

	.next .body {
		margin-top: var(--cds-spacing-06);
	}
</style>
