<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import InvitationList from '#lib/InvitationList.svelte';
	import OrganisationList from '#lib/OrganisationList.svelte';
	import Button from '#lib/ui/Button.svelte';
	import Checkbox from '#lib/ui/Checkbox.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import LoginForm from '#lib/ui/LoginForm.svelte';
	import Select from '#lib/ui/Select.svelte';
	import TextInput from '#lib/ui/TextInput.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let name = $state('');
	let timeZone = $state('');
	let accepted = $state(false);
	// Errors show once the field is left or the form is sent.
	let nameTouched = $state(false);
	let sent = $state(false);
	// Shown while the form is on its way, to Google or to create the
	// organisation. Reset when the browser comes back to this page from its
	// cache, e.g. after cancelling at Google.
	let leaving = $state(false);

	const nameError = $derived(
		(nameTouched || sent) && !name.trim() ? 'Organisation name is required' : undefined
	);
	const termsError = $derived(
		sent && !accepted ? 'Accept the terms and the privacy policy to continue' : undefined
	);
	const zoneOptions = $derived(data.time_zones.map((zone) => ({ value: zone, label: zone })));

	// The browser's time zone, known only in the browser.
	onMount(() => {
		const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;
		if (!timeZone && data.time_zones.includes(zone)) timeZone = zone;
	});

	function create(event: SubmitEvent) {
		sent = true;
		if (!name.trim() || !timeZone || !accepted || leaving) {
			event.preventDefault();
			const form = event.currentTarget as HTMLFormElement;
			form.querySelector<HTMLElement>('[aria-invalid="true"], select:invalid')?.focus();
			return;
		}
		leaving = true;
	}

	function go(event: SubmitEvent) {
		if (leaving) event.preventDefault();
		leaving = true;
	}
</script>

<svelte:window onpageshow={() => (leaving = false)} />

{#snippet problem()}
	<!-- Server errors just above the button (Carbon's notification placement). -->
	{#if data.error === 'failed'}
		<InlineNotification kind="error" title="Couldn't sign up">
			Something went wrong. Try again.
		</InlineNotification>
	{:else if data.error === 'invalid'}
		<InlineNotification kind="error" title="Couldn't create the organisation">
			Check the name and time zone, and accept the terms.
		</InlineNotification>
	{:else if data.error === 'unverified'}
		<InlineNotification kind="error" title="Couldn't sign up">
			Your Google account's email address isn't verified. Verify it with Google, then try again.
		</InlineNotification>
	{/if}
{/snippet}

{#if data.waiting}
	<LoginForm title="You're on the waiting list" description="We'll email you when there's room.">
		{#if !data.loggedIn}
			<p class="switch">
				Already have an account? <a href={resolve('/login')}>Log in</a>
			</p>
		{/if}
	</LoginForm>
{:else if data.adminOf}
	<!-- An admin can't create another (user's decision, 2026-11-16). -->
	<LoginForm
		title="You already have an organisation"
		description="You're the admin of {data.adminOf.name}."
	>
		<OrganisationList organisations={data.organisations} />
	</LoginForm>
{:else if data.full}
	<!-- Only the waiting list while full (user's decision, 2026-11-16). -->
	<LoginForm
		title="Sign-up is full for now"
		description={data.loggedIn
			? "Join the waiting list and we'll email you when there's room."
			: "Join the waiting list with your Google account and we'll email you when there's room."}
	>
		<!-- A native form: the backend sends the browser on to Google. -->
		<form method="post" action="/auth/signup" onsubmit={go}>
			{@render problem()}
			<Button type="submit" fill aria-disabled={leaving}>
				{leaving ? 'Joining…' : 'Join the waiting list'}
			</Button>
		</form>
		{#if !data.loggedIn}
			<p class="switch">
				Already have an account? <a href={resolve('/login')}>Log in</a>
			</p>
		{/if}
	</LoginForm>
{:else if !data.loggedIn}
	<LoginForm title="Create an organisation" description="Free, with your Google account.">
		<form method="post" action="/auth/signup" onsubmit={go}>
			{@render problem()}
			<Button type="submit" fill aria-disabled={leaving}>
				{leaving ? 'Signing up…' : 'Sign up with Google'}
			</Button>
		</form>
		<p class="switch">
			Already have an account? <a href={resolve('/login')}>Log in</a>
		</p>
	</LoginForm>
{:else}
	<LoginForm title="Name your organisation" description="You'll be its admin.">
		<div class="parts">
			{#if data.invitations.length}
				<!-- Before naming one, so nobody creates an organisation by mistake
				     when invited to one (user, 2026-11-17). -->
				<InvitationList invitations={data.invitations} title="Invitations" back="/signup" />
			{/if}
			<form method="post" action="/auth/signup/organisation" novalidate onsubmit={create}>
				<div class="fields">
					<TextInput
						label="Organisation name"
						name="organisation_name"
						bind:value={name}
						error={nameError}
						maxlength={100}
						required
						autocomplete="organization"
						onblur={() => (nameTouched = true)}
					/>
					<Select
						label="Time zone"
						name="time_zone"
						placeholder="Choose a time zone"
						options={zoneOptions}
						bind:value={timeZone}
						error={sent && !timeZone ? 'Time zone is required' : undefined}
						required
					/>
					<Checkbox name="terms" bind:checked={accepted} required error={termsError}>
						<!-- eslint-disable svelte/no-navigation-without-resolve -- external documents -->
						I accept the
						<a href={data.terms_url} target="_blank" rel="noopener noreferrer"
							>terms<span class="visually-hidden"> (opens in a new tab)</span></a
						>
						and the
						<a href={data.privacy_url} target="_blank" rel="noopener noreferrer"
							>privacy policy<span class="visually-hidden"> (opens in a new tab)</span></a
						>
						<!-- eslint-enable svelte/no-navigation-without-resolve -->
					</Checkbox>
				</div>
				{@render problem()}
				<Button type="submit" fill aria-disabled={leaving}>
					{leaving ? 'Creating…' : 'Create organisation'}
				</Button>
			</form>
			{#if data.organisations.length}
				<OrganisationList organisations={data.organisations} title="Your organisations" />
			{/if}
		</div>
	</LoginForm>
{/if}

<style>
	form {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	/* Closer together than the form's parts (my choice; Carbon is silent). */
	.fields {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-06);
	}

	/* Two groups: 32px between the form and the list (carbon-expert's
	   suggestion; Carbon gives no value). */
	.parts {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}

	.switch {
		margin: 0;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}
</style>
