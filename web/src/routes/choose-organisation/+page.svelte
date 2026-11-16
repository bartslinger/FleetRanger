<script lang="ts">
	import OrganisationList from '#lib/OrganisationList.svelte';
	import LoginForm from '#lib/ui/LoginForm.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<LoginForm title="Choose an organisation" description="You have access to more than one.">
	<OrganisationList organisations={data.organisations} returnTo={data.returnTo} />
</LoginForm>
