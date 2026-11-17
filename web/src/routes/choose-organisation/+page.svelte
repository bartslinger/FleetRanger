<script lang="ts">
	import InvitationList from '#lib/InvitationList.svelte';
	import OrganisationList from '#lib/OrganisationList.svelte';
	import Button from '#lib/ui/Button.svelte';
	import LoginForm from '#lib/ui/LoginForm.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const invited = $derived(data.invitations.length > 0);
	const title = $derived(
		data.organisations.length === 0
			? invited
				? 'Your invitations'
				: 'No organisations'
			: 'Choose an organisation'
	);
	const description = $derived(
		data.organisations.length === 0
			? invited
				? 'Accept one to get in.'
				: "You're not in any organisation."
			: invited
				? 'Or answer an invitation.'
				: data.organisations.length > 1
					? 'You have access to more than one.'
					: 'Open it to go on.'
	);
</script>

<LoginForm {title} {description}>
	<div class="parts">
		{#if invited}
			<InvitationList
				invitations={data.invitations}
				title={data.organisations.length ? 'Invitations' : undefined}
				returnTo={data.returnTo}
				back={data.back}
			/>
		{/if}
		{#if data.organisations.length}
			<OrganisationList
				organisations={data.organisations}
				title={invited ? 'Your organisations' : undefined}
				returnTo={data.returnTo}
			/>
		{:else}
			<!-- The way out when there is nowhere to go (user, 2026-11-17); a
			     ghost button, as it is an action (carbon-expert's suggestion). -->
			<form method="post" action="/auth/logout">
				<Button kind="ghost" type="submit">Log out</Button>
			</form>
		{/if}
	</div>
</LoginForm>

<style>
	/* 16px between the lists (carbon-expert's advice). */
	.parts {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
	}

	form {
		display: flex;
	}
</style>
