<script lang="ts">
	import Launch from '@carbon/icons/es/launch/16.js';
	import Drone from '@carbon/icons/es/drone/32.js';
	import Map from '@carbon/icons/es/map/32.js';
	import Mobile from '@carbon/icons/es/mobile/32.js';
	import Notebook from '@carbon/icons/es/notebook/32.js';
	import Button from '$ui/Button.svelte';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Icon from '$ui/Icon.svelte';
	import Tag from '$ui/Tag.svelte';
	import { CONTACT, SOURCE } from '#lib/site.js';

	const steps = [
		{
			title: 'Pick a pilot and fly',
			text: 'The FleetRanger app on the controller asks who is flying, also without a connection, and opens DJI Pilot 2.'
		},
		{
			title: 'Logs upload by themselves',
			text: 'After landing, the controller uploads new flight logs to your server, over the networks you allow.'
		},
		{
			title: 'Review your sessions',
			text: 'Flights are grouped into sessions, with their times, pilots, drones and tracks on a map.'
		}
	];

	const features = [
		{
			icon: Mobile,
			title: 'An app on the controller',
			text: 'Pilots choose their name from the list before each flight, also offline. Nobody logs in on the shared controller.'
		},
		{
			icon: Notebook,
			title: 'Logbook and sessions',
			text: 'Every flight with its time in the air, distance, drone, controller and pilot, grouped into sessions you can merge and split.'
		},
		{
			icon: Map,
			title: 'Tracks on a map',
			text: "Each flight's track on a satellite map, and all tracks of a session together."
		},
		{
			icon: Drone,
			title: 'Teams and equipment',
			text: 'Organise pilots in teams. Drones and controllers are registered from their logs, so the fleet list keeps itself up to date.'
		}
	];

	const privacy = [
		'One install per organisation, on your own server.',
		'Pilots see only their own flights, unless you give them more access.',
		'Raw logs and full telemetry are never shown to users.',
		'Open source under the AGPL-3.0, so you can check what it does.'
	];
</script>

<svelte:head>
	<title>FleetRanger: flight logs for DJI Enterprise fleets, on your own server</title>
	<meta
		name="description"
		content="FleetRanger collects the flight logs of your DJI Enterprise fleet and turns them into a logbook, sessions and maps, on your own server."
	/>
</svelte:head>

<section class="hero">
	<Grid centered rowGap>
		<Column md={8} lg={8}>
			<h1>Flight logs for DJI Enterprise fleets, on your own server</h1>
			<p class="lead">
				FleetRanger collects the flight logs from your controllers and turns them into a logbook,
				sessions and maps. It runs on your premises, so your flight data stays under your control.
			</p>
			<ul class="tags" aria-label="About FleetRanger">
				<li><Tag>Self-hosted</Tag></li>
				<li><Tag>Open source</Tag></li>
				<li><Tag>AGPL-3.0</Tag></li>
			</ul>
			<div class="actions">
				<Button href="mailto:{CONTACT}">Get in touch</Button>
				<Button href={SOURCE} kind="tertiary" icon={Launch}>View source code</Button>
			</div>
		</Column>
		<Column md={8} lg={8}>
			<div class="screenshot">Screenshot: a session with its tracks on a map</div>
		</Column>
	</Grid>
</section>

<section id="how-it-works" aria-labelledby="how-it-works-title">
	<Grid centered rowGap>
		<Column>
			<h2 id="how-it-works-title">How it works</h2>
		</Column>
		{#each steps as step, i (step.title)}
			<Column md={8} lg={5}>
				<div class="step">
					<span class="number" aria-hidden="true">{i + 1}</span>
					<h3><span class="visually-hidden">Step {i + 1}: </span>{step.title}</h3>
					<p>{step.text}</p>
				</div>
			</Column>
		{/each}
	</Grid>
</section>

<section id="features" aria-labelledby="features-title">
	<Grid centered rowGap>
		<Column>
			<h2 id="features-title">Features</h2>
		</Column>
		{#each features as feature (feature.title)}
			<Column md={4} lg={8}>
				<div class="tile">
					<Icon icon={feature.icon} />
					<h3>{feature.title}</h3>
					<p>{feature.text}</p>
				</div>
			</Column>
		{/each}
	</Grid>
</section>

<section id="privacy" class="band theme-dark" aria-labelledby="privacy-title">
	<Grid centered rowGap>
		<Column md={8} lg={8}>
			<h2 id="privacy-title">Your flight data stays on your server</h2>
			<p class="lead">
				Flight logs show where and when your teams operate. FleetRanger keeps them on your premises,
				where only the people you allow can see them.
			</p>
		</Column>
		<Column md={8} lg={8}>
			<ul class="checks">
				{#each privacy as point (point)}
					<li>{point}</li>
				{/each}
			</ul>
			<p class="note">
				Maps load their images from Mapbox. Flight tracks are drawn in the browser and are never
				sent to Mapbox.
			</p>
		</Column>
	</Grid>
</section>

<section aria-labelledby="start-title">
	<Grid centered rowGap>
		<Column md={8} lg={8}>
			<h2 id="start-title">Run FleetRanger for your fleet</h2>
			<p class="lead">
				FleetRanger works with DJI Enterprise drones flown with DJI Pilot 2. Write to
				<a href="mailto:{CONTACT}">{CONTACT}</a> to set it up for your organisation.
			</p>
		</Column>
	</Grid>
</section>

<style>
	section {
		padding-block: var(--cds-spacing-10);
	}

	.hero {
		padding-block: var(--cds-spacing-12);
	}

	@media (min-width: 66rem) {
		section {
			padding-block: var(--cds-spacing-11);
		}
	}

	section[id] {
		scroll-margin-top: var(--cds-spacing-05);
	}

	h1 {
		font-size: var(--cds-fluid-heading-05-font-size);
		font-weight: var(--cds-fluid-heading-05-font-weight);
		line-height: var(--cds-fluid-heading-05-line-height);
		letter-spacing: var(--cds-fluid-heading-05-letter-spacing);
	}

	h2 {
		margin-bottom: var(--cds-spacing-05);
		font-size: var(--cds-fluid-heading-04-font-size);
		font-weight: var(--cds-fluid-heading-04-font-weight);
		line-height: var(--cds-fluid-heading-04-line-height);
		letter-spacing: var(--cds-fluid-heading-04-letter-spacing);
	}

	h3 {
		margin-bottom: var(--cds-spacing-03);
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

	.lead {
		margin-top: var(--cds-spacing-06);
		color: var(--cds-text-secondary);
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-03);
		margin-top: var(--cds-spacing-06);
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--cds-spacing-05);
		margin-top: var(--cds-spacing-08);
	}

	.screenshot {
		display: grid;
		place-items: center;
		aspect-ratio: 16 / 10;
		border: 1px solid var(--cds-border-subtle-01);
		background: var(--cds-layer-01);
		color: var(--cds-text-secondary);
		text-align: center;
	}

	.number {
		display: block;
		margin-bottom: var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-fluid-heading-05-font-size);
		font-weight: 300;
		line-height: 1;
	}

	.step p,
	.tile p {
		color: var(--cds-text-secondary);
	}

	/* Carbon's base tile: not interactive. */
	.tile {
		height: 100%;
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
	}

	.tile :global(svg) {
		margin-bottom: var(--cds-spacing-07);
	}

	/* On a dark page, the band stands out one layer up. */
	@media (prefers-color-scheme: dark) {
		.band {
			background: var(--cds-layer-01);
		}
	}

	.checks {
		display: grid;
		gap: var(--cds-spacing-05);
	}

	.checks li {
		padding-inline-start: var(--cds-spacing-06);
		border-inline-start: 2px solid var(--cds-support-success);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
	}

	.note {
		margin-top: var(--cds-spacing-07);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
	}

	@media (min-width: 66rem) {
		.lead {
			max-width: 40rem;
		}
	}
</style>
