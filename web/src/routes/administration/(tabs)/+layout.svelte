<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import PageHeader from '#lib/ui/PageHeader.svelte';
	import TabLinks from '#lib/ui/TabLinks.svelte';
	import { may } from '#lib/permissions.js';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	// Apps only while apps can connect at all, as the list was before.
	const tabsList = $derived([
		{ href: resolve('/administration/(tabs)'), label: 'General' },
		...(data.account?.apps && may(data.account, 'apps.manage', null)
			? [{ href: resolve('/administration/(tabs)/apps'), label: 'Apps' }]
			: [])
	]);
	// In a later page's breadcrumb the tab says where it leads.
	const onApps = $derived(page.url.pathname.replace(/\/$/, '').endsWith('/administration/apps'));
</script>

<PageHeader title="Administration" crumb={onApps ? 'Apps' : undefined}>
	{#snippet tabs()}<TabLinks label="Administration" tabs={tabsList} />{/snippet}
</PageHeader>

{@render children()}
