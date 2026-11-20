<script lang="ts">
	import Launch from '@carbon/icons/es/launch/20.js';
	import Column from '$ui/Column.svelte';
	import Grid from '$ui/Grid.svelte';
	import Link from '$ui/Link.svelte';
	import bart400 from '#lib/images/bart-400.webp';
	import bart800 from '#lib/images/bart-800.webp';
	import { CONTACT } from '#lib/site.js';
</script>

<svelte:head>
	<title>About · FleetRanger</title>
	<meta
		name="description"
		content="Who makes FleetRanger, and why: flight logging for drone teams, on their own server."
	/>
</svelte:head>

<section aria-labelledby="title">
	<Grid centered rowGap>
		<Column>
			<h1 id="title">About</h1>
		</Column>
		<!-- The portrait beside the text from md, above it on phones, at half
		     the width there (carbon-expert's suggestion). -->
		<Column md={3} lg={4}>
			<img
				class="portrait"
				src={bart800}
				srcset="{bart400} 400w, {bart800} 800w"
				sizes="(min-width: 66rem) 20rem, (min-width: 42rem) 30vw, 50vw"
				alt="Portrait of Bart Slinger, holding a drone."
				width="800"
				height="800"
			/>
		</Column>
		<Column md={5} lg={8}>
			<div class="text">
				<h2>Bart Slinger</h2>
				<p>
					I build software for drones, from flight control to the tools around it. My company,
					<a href="https://mapture.ai">Mapture B.V.</a>, makes FleetRanger and
					<a href="https://skysync.io">SkySync</a>: live video from drones for police, fire services
					and others, in Europe and around the world.
				</p>
				<p>
					FleetRanger started when I was working with a drone team and saw them do their monthly
					reports by hand, every month. I'd heard the same frustration from others: there was no
					good solution, or only cloud services, which aren't an option for sensitive flight data.
				</p>
				<p>
					So I built a flight logbook that fills itself and runs on your own server. It's open
					source, so you can keep using it whatever happens.
				</p>
				<ul class="links">
					<li>
						<Link href="https://www.linkedin.com/in/bart-slinger/" size="lg" icon={Launch}
							>LinkedIn</Link
						>
					</li>
					<li>
						<Link href="https://bartslinger.com" size="lg" icon={Launch}>Blog</Link>
					</li>
					<li><Link href="mailto:{CONTACT}" size="lg">{CONTACT}</Link></li>
				</ul>
			</div>
		</Column>
	</Grid>
</section>

<section class="ai" aria-labelledby="ai-title">
	<Grid centered rowGap>
		<Column><div class="rule"></div></Column>
		<Column md={3} lg={4}>
			<h2 id="ai-title">Built with AI</h2>
		</Column>
		<Column md={5} lg={8}>
			<p class="text">
				I build FleetRanger with AI, and I think that's worth being open about. AI can move super
				fast, but give it one prompt and you get the typical AI slop. My approach is to keep
				steering it in the right direction along the way, from the backend architecture to the user
				interface. That way all that power works for me, and the result is what I actually wanted.
				Only now it takes me days instead of months.
			</p>
		</Column>
	</Grid>
</section>

<style>
	section {
		padding-block: var(--cds-spacing-09);
	}

	section.ai {
		padding-bottom: var(--cds-spacing-10);
	}

	/* The section starts at a thin rule above its heading, as on the home page. */
	.rule {
		border-top: 1px solid var(--cds-border-subtle-00);
	}

	.ai h2 {
		font-size: var(--cds-fluid-heading-04-font-size);
		font-weight: var(--cds-fluid-heading-04-font-weight);
		line-height: var(--cds-fluid-heading-04-line-height);
		letter-spacing: var(--cds-fluid-heading-04-letter-spacing);
	}

	.ai p {
		margin-top: 0;
	}

	h1 {
		font-size: var(--cds-fluid-heading-05-font-size);
		font-weight: var(--cds-fluid-heading-05-font-weight);
		line-height: var(--cds-fluid-heading-05-line-height);
		letter-spacing: var(--cds-fluid-heading-05-letter-spacing);
	}

	/* Square, as Carbon's 1:1 image ratio; half the width on phones. */
	.portrait {
		display: block;
		width: 50%;
		height: auto;
		margin-top: var(--cds-spacing-05);
	}

	@media (min-width: 42rem) {
		.portrait {
			width: 100%;
		}
	}

	.text {
		max-width: 36rem;
	}

	@media (min-width: 42rem) {
		.text {
			margin-top: var(--cds-spacing-05);
		}
	}

	h2 {
		font-size: var(--cds-heading-03-font-size);
		font-weight: var(--cds-heading-03-font-weight);
		line-height: var(--cds-heading-03-line-height);
		letter-spacing: var(--cds-heading-03-letter-spacing);
	}

	p {
		margin-top: var(--cds-spacing-05);
		font-size: var(--cds-body-02-font-size);
		line-height: var(--cds-body-02-line-height);
		letter-spacing: var(--cds-body-02-letter-spacing);
	}

	.links {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--cds-spacing-03);
		margin-top: var(--cds-spacing-07);
	}
</style>
