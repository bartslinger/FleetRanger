<script lang="ts">
	import Launch from '@carbon/icons/es/launch/16.js';
	import Button from '$ui/Button.svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
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
			title: 'Tracks on a map',
			text: "Each flight's track on a satellite map, and all tracks of a session together."
		},
		{
			title: 'Teams and equipment',
			text: 'Pilots work in teams. Drones and controllers are registered from their logs, so the fleet list keeps itself up to date.'
		}
	];
</script>

<svelte:head>
	<title>FleetRanger: flight logs for DJI Enterprise fleets, on your own server</title>
	<meta
		name="description"
		content="FleetRanger collects the flight logs of your DJI Enterprise fleet and turns them into a logbook of sessions, pilots and tracks, on your own server."
	/>
</svelte:head>

<section class="hero" aria-labelledby="title">
	<Grid centered rowGap>
		<Column md={8} lg={7}>
			<h1 id="title">Own your flight logs</h1>
			<p class="intro">
				FleetRanger collects the logs from your DJI Enterprise controllers on your own server, and
				turns them into a logbook: every flight with its pilot and track, grouped into sessions. It
				is open source.
			</p>
			<div class="actions">
				<Button href="mailto:{CONTACT}">Get in touch</Button>
				<Button href={SOURCE} kind="tertiary" icon={Launch}>View source code</Button>
			</div>
		</Column>
		<Column md={8} lg={9}>
			<div class="hero-laptop">
				<Laptop
					{...session}
					width={40}
					alt="A session in FleetRanger: a flight's track on a satellite map, with its pilot, team, times and distance, and the list of its flights."
				/>
			</div>
		</Column>
	</Grid>
</section>

<section id="how-it-works" aria-labelledby="how-it-works-title">
	<Grid centered>
		<Column>
			<h2 id="how-it-works-title">How it works</h2>
		</Column>
		<Column>
			<!-- The steps scroll past the picture, which stays in place and shows the
			     step in the middle of the window (the script in app.html). -->
			<div class="steps" data-steps>
				<div class="picture">
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

<section id="features" aria-labelledby="features-title">
	<Grid centered rowGap>
		<Column md={8} lg={4}>
			<h2 id="features-title">Features</h2>
		</Column>
		<Column md={8} lg={12}>
			<ul class="blocks">
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

<section aria-labelledby="start-title">
	<Grid centered rowGap>
		<Column md={8} lg={4}>
			<h2 id="start-title">Set it up for your fleet</h2>
		</Column>
		<Column md={8} lg={8}>
			<p class="body">
				FleetRanger works with DJI Enterprise drones flown with DJI Pilot 2. To run it for your
				organisation, write to {CONTACT}.
			</p>
			<div class="actions">
				<Button href="mailto:{CONTACT}" kind="tertiary">Email us</Button>
			</div>
		</Column>
	</Grid>
</section>

<style>
	section {
		padding-block: var(--cds-spacing-10);
	}

	@media (min-width: 66rem) {
		section {
			padding-block: var(--cds-spacing-11);
		}
	}

	section[id] {
		scroll-margin-top: var(--cds-spacing-05);
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
		margin-top: var(--cds-spacing-07);
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

	/* The picture stays in place while the steps scroll past it: at the top on
	   phones, beside them from md. */
	.steps {
		display: grid;
		margin-top: var(--cds-spacing-09);
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
			top: calc(50vh - 12rem);
			padding-block: 0;
			background: none;
		}

		.shown {
			max-width: 32rem;
		}
	}

	/* Three moments along a line, marked like a track: a ring at take-off,
	   dots after it. Each is tall enough to be read while its picture shows. */
	.timeline {
		display: grid;
		gap: var(--cds-spacing-07);
	}

	.timeline li {
		position: relative;
		min-height: 50vh;
		padding-inline-start: var(--cds-spacing-08);
	}

	@media (min-width: 42rem) {
		.timeline {
			grid-column: 1;
			grid-row: 1;
			padding-top: calc(50vh - 12rem);
		}

		.timeline li {
			min-height: 24rem;
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
		bottom: calc(-1 * var(--cds-spacing-07) + 0.25rem);
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

	/* Beside the headline from lg, below the buttons before that. */
	.hero-laptop {
		max-width: 40rem;
		margin-top: var(--cds-spacing-07);
	}

	@media (min-width: 66rem) {
		.hero-laptop {
			margin-top: 0;
		}
	}

	/* Carbon's content blocks: a title and a short text, separated by space. */
	.blocks {
		display: grid;
		gap: var(--cds-spacing-08) var(--cds-spacing-07);
	}

	@media (min-width: 42rem) {
		.blocks {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	.blocks p {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}
</style>
