<script lang="ts">
	import '#lib/ui/base.css';
	import { dev } from '$app/env';
	import { resolve } from '$app/paths';
	import favicon from '#lib/assets/favicon.svg';
	import Shell, { type NavItem } from '#lib/ui/Shell.svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	const nav: NavItem[] = [
		{ href: resolve('/'), label: 'Logbook' },
		...(dev ? [{ href: resolve('/design'), label: 'Components' }] : [])
	];
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>FleetRanger</title>
</svelte:head>

<Shell name="FleetRanger" {nav} theme={data.theme}>
	{@render children()}
</Shell>
