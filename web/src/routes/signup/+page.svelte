<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
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
	// Shown while the browser goes to Google. Reset when it comes back to
	// this page from the browser's cache, e.g. after cancelling there.
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

	function submit(event: SubmitEvent) {
		sent = true;
		if (!name.trim() || !timeZone || !accepted || leaving) {
			event.preventDefault();
			const form = event.currentTarget as HTMLFormElement;
			form.querySelector<HTMLElement>('[aria-invalid="true"], select:invalid')?.focus();
			return;
		}
		leaving = true;
	}
</script>

<svelte:window onpageshow={() => (leaving = false)} />

<LoginForm title="Create an organisation" description="Free, with your Google account.">
	{#if data.waiting}
		<InlineNotification kind="info" title="You're on the waiting list">
			Sign-up is full for now. We'll email you when there's room.
		</InlineNotification>
	{:else}
		{#if data.full}
			<InlineNotification kind="info" title="Sign-up is full">
				Continue to join the waiting list. We'll email you when there's room.
			</InlineNotification>
		{/if}
		<!-- A native form: the backend sends the browser on to Google. -->
		<form method="post" action="/auth/signup" novalidate onsubmit={submit}>
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
			<!-- Server errors just above the button (Carbon's notification placement). -->
			{#if data.error === 'failed'}
				<InlineNotification kind="error" title="Couldn't create the organisation">
					Something went wrong. Try again.
				</InlineNotification>
			{:else if data.error === 'invalid'}
				<InlineNotification kind="error" title="Couldn't create the organisation">
					Check the name and time zone, and accept the terms.
				</InlineNotification>
			{:else if data.error === 'expired'}
				<InlineNotification kind="error" title="Couldn't create the organisation">
					This took too long. Fill in the form again.
				</InlineNotification>
			{:else if data.error === 'unverified'}
				<InlineNotification kind="error" title="Couldn't create the organisation">
					Your Google account's email address isn't verified. Verify it with Google, then try again.
				</InlineNotification>
			{/if}
			<Button type="submit" fill aria-disabled={leaving}>
				{leaving ? 'Creating…' : 'Create with Google'}
			</Button>
		</form>
	{/if}
	<p class="switch">
		Already have an account? <a href={resolve('/login')}>Log in</a>
	</p>
</LoginForm>

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

	.switch {
		margin: 0;
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}
</style>
