<script lang="ts">
	import Prose from '#lib/Prose.svelte';
	import { APP, COMPANY, CONTACT, LEGAL_UPDATED, SOURCE } from '#lib/site.js';
</script>

<svelte:head>
	<meta name="description" content="The terms for the hosted FleetRanger at {APP}." />
</svelte:head>

<Prose title="Terms of use" updated={LEGAL_UPDATED}>
	<p>
		These terms apply to the hosted FleetRanger at {APP}, which {COMPANY.name} offers. By creating an
		organisation there you agree to them, on behalf of the business or institution you act for.
	</p>
	<p>
		They don't cover FleetRanger on your own server. That is the same software under its
		<a href="https://www.gnu.org/licenses/agpl-3.0.html">open-source licence</a>, without any
		agreement with us.
	</p>

	<h2>1. Who we are</h2>
	<p>
		{COMPANY.name}, in the Netherlands, registered with the Dutch Chamber of Commerce under number
		{COMPANY.kvk}, VAT number {COMPANY.vat}. Write to us at
		<a href="mailto:{CONTACT}">{CONTACT}</a>.
	</p>

	<h2>2. Not for consumers</h2>
	<p>
		The service is for businesses, public bodies and other institutions. It is not for consumers. By
		creating an organisation you confirm that you act for one of these, and that you may agree to
		these terms on its behalf.
	</p>
	<p>
		In these terms "you" is that business or institution, and your "organisation" is its account in
		FleetRanger.
	</p>

	<h2>3. Your organisation and its users</h2>
	<p>
		You log in with an account at a login provider the service supports, such as Google. Whoever
		creates an organisation is its first admin. Admins decide who else gets access, and you are
		responsible for what your users do in the service.
	</p>

	<h2>4. Plans and limits</h2>
	<p>
		The service comes in plans, each with its limits and its price, if it has one. You pay nothing
		unless you choose a paid plan, and you see its price and how you pay before you choose. We can
		change plans, limits and prices, as section 11 describes.
	</p>
	<p>
		We do our best to keep the service working, but we don't promise that it is always available.
		Support is included only where a plan says so.
	</p>

	<h2>5. Your data</h2>
	<p>
		Your flight logs and everything else you put into the service stay yours. We use them only to
		provide the service to you.
	</p>
	<p>
		The logs contain personal data, such as who flew where and when. For that data you are the
		controller and we are your processor. The
		<a href="/processing-agreement">processing agreement</a> says what that means, and it is part of these
		terms.
	</p>
	<p>
		You make sure that you may upload your logs, and that your pilots know their flights are
		recorded here.
	</p>

	<h2>6. What isn't allowed</h2>
	<ul>
		<li>Uploading logs or other data that you have no right to use.</li>
		<li>Using the service for anything unlawful.</li>
		<li>Harming the service or its other users.</li>
	</ul>
	<p>
		We can block or close your organisation if you do any of this, at once and without the notice of
		section 10.
	</p>

	<h2>7. The software and other services</h2>
	<p>
		FleetRanger is open source. Its <a href={SOURCE}>source code</a> shows what it does with your data.
	</p>
	<p>
		The service depends on others: DJI gives the keys needed to read a flight log, a login provider
		such as Google handles the login, and Mapbox supplies the maps. We are not affiliated with DJI.
		When one of these services changes or stops, parts of FleetRanger may stop working.
	</p>

	<h2>8. No guarantees</h2>
	<p>
		The service comes as it is. FleetRanger calculates its figures from your logs with care, but
		they can be wrong or incomplete. Check them before you rely on them, for example in a report to
		an aviation authority.
	</p>
	<p>
		Your controllers keep the original flight logs. Keep them there, or keep copies, for as long as
		you need them.
	</p>

	<h2>9. Liability</h2>
	<p>
		We are not liable for damage that comes from using the service or from not being able to use it,
		such as lost data, lost income or a missed deadline. This doesn't apply to damage caused by our
		intent or deliberate recklessness.
	</p>

	<h2>10. Ending</h2>
	<p>
		You can stop at any time: an admin deletes your organisation on its Administration page, which
		removes it with everything in it at once.
	</p>
	<p>
		We can end the service, or close your organisation, with 30 days' notice by email to your
		admins. Then we delete its data.
	</p>
	<p>Deleted data stays in our backups for at most 30 days more.</p>

	<h2>11. Changes</h2>
	<p>
		We can change these terms, the processing agreement, and the plans with their limits and prices.
		We announce a change by email to your admins, at least 30 days ahead. If you don't agree, delete
		your organisation before the change takes effect. Using the service after that date means
		accepting the change.
	</p>
	<p>
		Our emails go to the email addresses your admins log in with, so make sure someone reads them.
	</p>

	<h2>12. Law and court</h2>
	<p>
		Dutch law applies to these terms. Disputes go to the competent court in the district where
		{COMPANY.name} has its registered office.
	</p>
</Prose>
