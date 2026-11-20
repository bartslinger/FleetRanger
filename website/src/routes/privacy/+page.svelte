<script lang="ts">
	import Prose from '#lib/Prose.svelte';
	import { APP, COMPANY, CONTACT, LEGAL_UPDATED } from '#lib/site.js';
</script>

<svelte:head>
	<meta
		name="description"
		content="What {COMPANY.name} does with personal data on fleetranger.org and {APP}."
	/>
</svelte:head>

<Prose title="Privacy policy" updated={LEGAL_UPDATED}>
	<p>
		This policy says what {COMPANY.name} does with personal data on this website and in the hosted FleetRanger
		at {APP}. It doesn't cover FleetRanger on someone's own server; whoever runs that server is
		responsible for it.
	</p>

	<h2>1. Who is responsible</h2>
	<p>
		{COMPANY.name}, in the Netherlands, Chamber of Commerce number {COMPANY.kvk}. Write to us at
		<a href="mailto:{CONTACT}">{CONTACT}</a>.
	</p>

	<h2>2. Flight data belongs to your organisation</h2>
	<p>
		The flights, names and other data inside an organisation in FleetRanger are that organisation's.
		It decides what happens with them, and we only store and process them for it, under our <a
			href="/processing-agreement">processing agreement</a
		>. If you are a pilot or a user and have a question about that data, ask your organisation.
	</p>
	<p>The rest of this policy is about the data we are responsible for ourselves.</p>

	<h2>3. What we keep, why and how long</h2>
	<ul>
		<li>
			<strong>Your login.</strong> When you log in, the login provider you use, such as Google, gives
			us your name, your email address and an identifier of your account there. We use them to recognise
			you and to connect you to your organisation. We keep them until your login is removed or your organisation
			is deleted.
		</li>
		<li>
			<strong>Your login session.</strong> A cookie keeps you logged in. It ends when you log out, or
			after 30 days without use.
		</li>
		<li>
			<strong>The waiting list.</strong> If you join it, we keep your name and email address to tell you
			when there is room. We keep them until you have a place or ask us to remove you.
		</li>
		<li>
			<strong>Server logs.</strong> Our servers record each request: the IP address, the time, the page
			and the browser. We use that to keep the service secure and to find faults, and delete it after
			90 days.
		</li>
		<li>
			<strong>Email.</strong> When you write to us, we keep the conversation for as long as it takes to
			deal with it. We email an organisation's admins about the service, for example about a change to
			the terms.
		</li>
		<li>
			<strong>Visits to this website.</strong> We count visits with Plausible. It sets no cookies and
			keeps no data about you as a person, only totals, such as how often a page was viewed and from which
			country.
		</li>
	</ul>
	<p>
		We do this because we need it to provide the service to you and your organisation, and because
		we have a legitimate interest in keeping it secure and in knowing how the website is used.
	</p>

	<h2>4. Cookies</h2>
	<p>
		The hosted FleetRanger sets one cookie, for your login session. The service doesn't work without
		it. This website sets no cookies; it remembers your choice of a light or dark theme in your
		browser only. There are no advertising or tracking cookies.
	</p>

	<h2>5. Who else receives data</h2>
	<p>We don't sell data and share it only with the companies we need to run the service:</p>
	<table>
		<thead>
			<tr>
				<th scope="col">Company</th>
				<th scope="col">What for</th>
				<th scope="col">What it receives</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td>Amazon Web Services</td>
				<td>Servers and storage, in Frankfurt, Germany</td>
				<td>Everything the service stores</td>
			</tr>
			<tr>
				<td>Hetzner</td>
				<td>Monitoring the service, in Falkenstein, Germany</td>
				<td>The server logs</td>
			</tr>
			<tr>
				<td>Google</td>
				<td>Logging in with a Google account</td>
				<td>That you log in to FleetRanger, and when</td>
			</tr>
			<tr>
				<td>Mapbox</td>
				<td>Maps in the service</td>
				<td>Your IP address and roughly which area you look at; never the flight paths</td>
			</tr>
			<tr>
				<td>Plausible</td>
				<td>Counting visits to this website</td>
				<td>The page you visit; your IP address is used briefly and not kept</td>
			</tr>
			<tr>
				<td>GitHub</td>
				<td>Hosting this website</td>
				<td>Your IP address and the pages you request</td>
			</tr>
		</tbody>
	</table>
	<p>
		Google, Mapbox and GitHub are American companies and may process this data in the United States,
		under the EU–US Data Privacy Framework or the European Commission's standard contractual
		clauses. What the service stores stays in the European Union.
	</p>

	<h2>6. Your rights</h2>
	<p>
		You can ask us to show you the data we keep about you, to correct or delete it, to limit what we
		do with it, or to give it to you in a file. You can also object to our using it. Write to
		<a href="mailto:{CONTACT}">{CONTACT}</a> and we answer within a month.
	</p>
	<p>
		If you are not satisfied with how we handle your data, you can complain to the Dutch data
		protection authority, the
		<a href="https://www.autoriteitpersoonsgegevens.nl">Autoriteit Persoonsgegevens</a>.
	</p>

	<h2>7. Changes</h2>
	<p>
		When this policy changes, the date at the top changes with it. We announce important changes by
		email to the admins of the organisations in the service.
	</p>
</Prose>
