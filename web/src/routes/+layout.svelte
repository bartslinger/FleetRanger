<script lang="ts">
	import '#lib/ui/base.css';
	import Events from '@carbon/icons/es/events/16.js';
	import Notebook from '@carbon/icons/es/notebook/16.js';
	import Template from '@carbon/icons/es/template/16.js';
	import UserMultiple from '@carbon/icons/es/user--multiple/16.js';
	import { dev } from '$app/env';
	import { resolve } from '$app/paths';
	import favicon from '#lib/assets/favicon.svg';
	import Shell, { type NavItem } from '#lib/ui/Shell.svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	// Admin pages only for admins; the backend refuses everyone else anyway.
	const nav: NavItem[] = $derived([
		{ href: resolve('/'), label: 'Logbook', icon: Notebook },
		...(data.account?.role === 'admin'
			? [
					{ href: resolve('/pilots'), label: 'Pilots', icon: UserMultiple },
					{ href: resolve('/teams'), label: 'Teams', icon: Events }
				]
			: []),
		...(dev ? [{ href: resolve('/design'), label: 'Components', icon: Template }] : [])
	]);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>FleetRanger</title>
</svelte:head>

<Shell name="FleetRanger" {nav} account={data.account} theme={data.theme}>
	{@render children()}
</Shell>
