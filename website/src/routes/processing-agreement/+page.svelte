<script lang="ts">
	import Prose from '#lib/Prose.svelte';
	import { APP, COMPANY, CONTACT, LEGAL_UPDATED, SOURCE } from '#lib/site.js';
</script>

<svelte:head>
	<meta
		name="description"
		content="How {COMPANY.name} handles the personal data in your flight logs on {APP}."
	/>
</svelte:head>

<Prose title="Processing agreement" updated={LEGAL_UPDATED}>
	<p>
		Your flight logs contain personal data. Under the General Data Protection Regulation (GDPR) you
		are its controller, and {COMPANY.name}, which runs {APP}, is your processor. As in the terms,
		"you" is the business or institution that uses the service, and your "organisation" is its
		account. This agreement says how we handle that data. It is part of the
		<a href="/terms">terms of use</a>, so it applies from the moment your organisation is created.
	</p>

	<h2>1. What we process</h2>
	<ul>
		<li>
			<strong>What for:</strong> to provide the service: storing your flight logs, reading them, and showing
			your users the flights, sessions and figures that come out of them.
		</li>
		<li><strong>How long:</strong> for as long as your organisation exists in the service.</li>
		<li>
			<strong>Whose data:</strong> your pilots and the other people you add as users.
		</li>
		<li>
			<strong>Which data:</strong> names; the email addresses of users who log in and the identifiers
			of the accounts they log in with; flights with their times, places and flight paths, and who flew
			them; the serial numbers of drones, controllers and batteries; comments your users write; and the
			controller app's error reports, with the controller's model, Android version and id in the
			service, kept for 90 days.
		</li>
	</ul>

	<h2>2. Only on your instructions</h2>
	<p>
		We process the data only to provide the service as the terms and this agreement describe, and as
		your users direct it through the service. Those are your instructions. We use the data for
		nothing else, unless a law requires it; then we tell you first, if the law allows.
	</p>
	<p>If we think an instruction breaks the law, we tell you.</p>

	<h2>3. Who can see your data</h2>
	<p>
		We look into your organisation's data only when that is needed to keep the service working and
		secure, or when you ask us to. Everyone who can reach the data on our side is bound to keep it
		confidential.
	</p>

	<h2>4. Security</h2>
	<p>We protect the data with at least these measures:</p>
	<ul>
		<li>All connections to the service are encrypted (HTTPS).</li>
		<li>The database and the stored flight logs are encrypted on disk.</li>
		<li>Everything is kept in the European Union, in Germany.</li>
		<li>The database keeps each organisation's data apart from the others'.</li>
		<li>Only the people who run the service can reach its servers.</li>
		<li>A backup is made every day and kept for 30 days.</li>
		<li>
			The <a href={SOURCE}>source code</a> is public, so anyone can check what the service does.
		</li>
	</ul>
	<p>We keep these measures up to date as the service and the risks change.</p>

	<h2>5. Sub-processors</h2>
	<p>We use these other companies to process your data, and you agree to that:</p>
	<table>
		<thead>
			<tr>
				<th scope="col">Company</th>
				<th scope="col">What for</th>
				<th scope="col">Where</th>
			</tr>
		</thead>
		<tbody>
			<tr>
				<td>Amazon Web Services EMEA SARL</td>
				<td>Servers, database and storage of flight logs</td>
				<td>Frankfurt, Germany</td>
			</tr>
			<tr>
				<td>Hetzner Online GmbH</td>
				<td>Monitoring the service: server logs and the controller app's error reports</td>
				<td>Falkenstein, Germany</td>
			</tr>
		</tbody>
	</table>
	<p>
		We have an agreement with each of them that protects your data as this one does, and we remain
		responsible to you for what they do.
	</p>
	<p>
		Before we add or replace a sub-processor, we tell your admins by email, at least 30 days ahead.
		If you object, tell us. If we can't resolve it, you can delete your organisation before the
		change.
	</p>

	<h2>6. Outside the European Economic Area</h2>
	<p>
		We store and process your data in the European Union. We don't move it outside the European
		Economic Area, unless we announce that as a change of sub-processors and the GDPR's rules for
		such transfers are met.
	</p>

	<h2>7. Requests from people</h2>
	<p>
		A pilot or another user may ask to see, correct or delete their data, or to receive it.
		Answering is up to you. You do that with what the service offers: your admins can see a person's
		flights, change their name, and archive or delete them.
	</p>
	<p>
		If someone sends such a request to us, we pass it on to your admins and don't answer it
		ourselves. We don't handle requests for you.
	</p>
	<p>
		If you need to know more about how the service works, for example to assess its risks for the
		people in your logs, ask us.
	</p>

	<h2>8. Data breaches</h2>
	<p>
		If your data is leaked, lost or seen by someone who shouldn't, we tell your admins by email
		without undue delay: what happened, which data it concerns, and what we have done and will do
		about it.
	</p>

	<h2>9. When it ends</h2>
	<p>
		When your organisation is deleted, by you or by us under the terms, all its data is removed from
		the service at once. It stays in our backups for at most 30 days more, and is not used there.
	</p>
	<p>
		Take out what you want to keep before that, with what the service offers. The original flight
		logs stay on your own controllers.
	</p>

	<h2>10. Checking us</h2>
	<p>
		We answer your reasonable questions about how we keep to this agreement. If the answers and the
		public source code are not enough, you may have an independent auditor check it, at most once a
		year and at a time we agree on together. You pay the auditor, and our reasonable costs for the
		time it takes us.
	</p>

	<h2>11. Contact</h2>
	<p>
		For everything about this agreement, write to
		<a href="mailto:{CONTACT}">{CONTACT}</a>. The terms of use also apply to this agreement,
		including their sections on liability, changes and Dutch law.
	</p>
</Prose>
