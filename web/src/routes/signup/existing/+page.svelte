<script lang="ts">
	import OrganisationList from '#lib/OrganisationList.svelte';
	import Button from '#lib/ui/Button.svelte';
	import LoginForm from '#lib/ui/LoginForm.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let creating = $state(false);
</script>

<svelte:window onpageshow={() => (creating = false)} />

<!-- One page for both cases: create, or open an organisation of theirs (user's
     design, 2026-11-16). An admin can't create another (user's decision). -->
{#if data.adminOf}
	<LoginForm
		title="You already have an organisation"
		description="You're the admin of {data.adminOf.name}."
	>
		<OrganisationList organisations={data.organisations} />
	</LoginForm>
{:else}
	<LoginForm
		title="Create {data.organisationName}?"
		description="You'll be the new organisation's admin."
	>
		<div class="parts">
			<!-- The choices come from the login cookie, not from the form. -->
			<form
				method="post"
				action="/auth/signup/existing"
				onsubmit={(event) => {
					if (creating) event.preventDefault();
					creating = true;
				}}
			>
				<Button type="submit" fill aria-disabled={creating}>
					{creating ? 'Creating…' : 'Create organisation'}
				</Button>
			</form>
			<OrganisationList organisations={data.organisations} title="Your organisations" />
		</div>
	</LoginForm>
{/if}

<style>
	/* Two groups: 32px between the button and the list (carbon-expert's
	   suggestion; Carbon gives no value). */
	.parts {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-07);
	}
</style>
