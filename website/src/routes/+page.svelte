<script lang="ts">
	import Launch from '@carbon/icons/es/launch/16.js';
	import Button from '$ui/Button.svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import FlightTrack from '#lib/FlightTrack.svelte';
	import { CONTACT, SOURCE } from '#lib/site.js';

	const moments = [
		{
			when: 'Before take-off',
			text: 'The pilot picks their name on the controller, even without a connection, and FleetRanger opens DJI Pilot 2.'
		},
		{
			when: 'After landing',
			text: 'The controller uploads the new flight logs to your server, over Wi-Fi networks you approve.'
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

	const privacy = [
		{
			title: 'One install per organisation',
			text: 'FleetRanger runs on your own server, and your flight logs are stored there only.'
		},
		{
			title: 'Pilots see their own flights',
			text: 'Pilots see only their own flights, unless you give them more access. Raw logs and full telemetry are never shown to users.'
		},
		{
			title: 'Open source',
			text: 'Licensed under the AGPL-3.0, so you can check what it does with your data.'
		}
	];

	const services = [
		{
			title: 'DJI, to read a log',
			text: "DJI encrypts flight logs. To read one, your server asks DJI for its keys, sending only the log's encrypted key records, not the flight."
		},
		{
			title: 'Mapbox, for the maps',
			text: "Map images come from Mapbox, which sees the area being viewed and the viewer's IP address. Flight tracks are drawn in the browser and are never sent."
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
		<Column md={8} lg={6}>
			<h1 id="title">Your flight logs stay on your own server</h1>
			<p class="intro">
				FleetRanger collects the logs from your DJI Enterprise controllers and turns them into a
				logbook: every flight with its pilot and track, grouped into sessions. It is self-hosted and
				open source.
			</p>
			<div class="actions">
				<Button href="mailto:{CONTACT}">Get in touch</Button>
				<Button href={SOURCE} kind="tertiary" icon={Launch}>View source code</Button>
			</div>
		</Column>
		<Column md={8} lg={10}>
			<FlightTrack />
		</Column>
	</Grid>
</section>

<section id="privacy" class="layer" aria-labelledby="privacy-title">
	<Grid centered rowGap>
		<Column md={8} lg={4}>
			<h2 id="privacy-title">Privacy</h2>
		</Column>
		<Column md={8} lg={12}>
			<ul class="blocks three">
				{#each privacy as point (point.title)}
					<li>
						<h3>{point.title}</h3>
						<p>{point.text}</p>
					</li>
				{/each}
			</ul>
			<h3 class="group">Two outside services</h3>
			<ul class="blocks">
				{#each services as service (service.title)}
					<li>
						<h4>{service.title}</h4>
						<p>{service.text}</p>
					</li>
				{/each}
			</ul>
		</Column>
	</Grid>
</section>

<section id="how-it-works" aria-labelledby="how-it-works-title">
	<Grid centered>
		<Column>
			<h2 id="how-it-works-title">How it works</h2>
		</Column>
		<Column>
			<ol class="timeline">
				{#each moments as moment, i (moment.when)}
					<li class:last={i === moments.length - 1}>
						<h3>{moment.when}</h3>
						<p>{moment.text}</p>
					</li>
				{/each}
			</ol>
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

	.layer {
		background: var(--cds-layer-01);
	}

	/* The headline is the page's voice: Carbon's light display style. */
	h1 {
		font-size: var(--cds-fluid-display-01-font-size);
		font-weight: var(--cds-fluid-display-01-font-weight);
		line-height: var(--cds-fluid-display-01-line-height);
		letter-spacing: var(--cds-fluid-display-01-letter-spacing);
	}

	h2 {
		font-size: var(--cds-fluid-heading-04-font-size);
		font-weight: var(--cds-fluid-heading-04-font-weight);
		line-height: var(--cds-fluid-heading-04-line-height);
		letter-spacing: var(--cds-fluid-heading-04-letter-spacing);
	}

	h3,
	h4 {
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

	/* Three moments along a line, marked like a track: a ring at take-off,
	   dots after it. Down the left edge on phones, across from md. */
	.timeline {
		display: grid;
		gap: var(--cds-spacing-07);
		margin-top: var(--cds-spacing-09);
	}

	.timeline li {
		position: relative;
		padding-inline-start: var(--cds-spacing-08);
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

	@media (min-width: 42rem) {
		.timeline {
			grid-template-columns: repeat(3, minmax(0, 1fr));
			gap: var(--cds-spacing-07);
		}

		.timeline li {
			padding-inline-start: 0;
			padding-top: var(--cds-spacing-08);
		}

		.timeline li::before {
			top: 0;
		}

		.timeline li::after {
			top: calc(0.4375rem - 1px);
			bottom: auto;
			left: 1.25rem;
			right: calc(-1 * var(--cds-spacing-07) + 0.25rem);
			border-left: 0;
			border-top: 2px solid var(--cds-border-subtle-01);
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

	@media (min-width: 66rem) {
		.three {
			grid-template-columns: repeat(3, minmax(0, 1fr));
		}
	}

	.blocks p {
		margin-top: var(--cds-spacing-03);
		color: var(--cds-text-secondary);
	}

	/* A group of blocks within the section: a smaller heading above them. */
	.group {
		margin-block: var(--cds-spacing-10) var(--cds-spacing-06);
		color: var(--cds-text-secondary);
		font-size: var(--cds-heading-02-font-size);
		font-weight: var(--cds-heading-02-font-weight);
		line-height: var(--cds-heading-02-line-height);
	}
</style>
