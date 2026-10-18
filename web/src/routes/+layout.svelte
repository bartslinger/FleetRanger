<script lang="ts">
	import '#lib/ui/base.css';
	import Drone from '@carbon/icons/es/drone/16.js';
	import Events from '@carbon/icons/es/events/16.js';
	import FlightRoster from '@carbon/icons/es/flight--roster/16.js';
	import Notebook from '@carbon/icons/es/notebook/16.js';
	import NotificationNew from '@carbon/icons/es/notification--new/20.js';
	import Template from '@carbon/icons/es/template/16.js';
	import UserMultiple from '@carbon/icons/es/user--multiple/16.js';
	import { dev } from '$app/env';
	import { invalidateAll } from '$app/navigation';
	import { resolve } from '$app/paths';
	import favicon from '#lib/assets/favicon.svg';
	import ProcessingPanel from '#lib/ProcessingPanel.svelte';
	import type { ProcessingStatus } from '#lib/server/backend.js';
	import Icon from '#lib/ui/Icon.svelte';
	import Loading from '#lib/ui/Loading.svelte';
	import Shell, { type HeaderAction, type NavItem } from '#lib/ui/Shell.svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	// Log processing, for admins: from the server, then polled; often while
	// logs are being processed, rarely otherwise (to notice new ones).
	let processing = $derived(data.processing);
	const BUSY_POLL_MS = 3_000;
	const IDLE_POLL_MS = 30_000;

	$effect(() => {
		if (!processing) return;
		const busy = processing.pending > 0;
		const timer = setTimeout(poll, busy ? BUSY_POLL_MS : IDLE_POLL_MS);
		return () => clearTimeout(timer);
	});

	async function poll() {
		const before = processing;
		let next: ProcessingStatus;
		try {
			const response = await fetch('/api/logs/processing');
			if (!response.ok) return;
			next = await response.json();
		} catch {
			return;
		}
		processing = next;
		// A batch finished: reload the page's data, so new flights show.
		if (before && before.pending > 0 && next.pending === 0) await invalidateAll();
	}

	// Controllers waiting to be paired, for everyone: from the server, then
	// polled, so someone at a controller sees it come up.
	let pairing = $derived(data.pairing);
	const PAIRING_POLL_MS = 15_000;

	$effect(() => {
		if (!data.account) return;
		const timer = setInterval(async () => {
			try {
				const response = await fetch('/api/pairing');
				if (response.ok) pairing = await response.json();
			} catch {
				// Offline for a moment: try again next time.
			}
		}, PAIRING_POLL_MS);
		return () => clearInterval(timer);
	});

	const actions: HeaderAction[] = $derived([
		...(pairing.length > 0
			? [
					{
						id: 'pairing',
						label:
							pairing.length === 1
								? 'A controller is waiting to be paired'
								: `${pairing.length} controllers are waiting to be paired`,
						icon: pairingIcon,
						href: resolve('/pair')
					}
				]
			: []),
		...(processing && processing.pending > 0
			? [
					{
						id: 'processing',
						label: `Processing logs: ${processing.processed} of ${processing.processed + processing.pending}`,
						icon: loadingIcon,
						panel: processingPanel
					}
				]
			: [])
	]);

	// Admin pages only for admins; the backend refuses everyone else anyway.
	// Equipment for everyone: members see their team's.
	const nav: NavItem[] = $derived([
		{ href: resolve('/'), label: 'Logbook', icon: Notebook },
		{ href: resolve('/sessions'), label: 'Sessions', icon: FlightRoster },
		...(data.account?.role === 'admin'
			? [
					{ href: resolve('/users'), label: 'Users', icon: UserMultiple },
					{ href: resolve('/teams'), label: 'Teams', icon: Events }
				]
			: []),
		{ href: resolve('/equipment'), label: 'Equipment', icon: Drone },
		...(dev ? [{ href: resolve('/design'), label: 'Components', icon: Template }] : [])
	]);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>FleetRanger</title>
</svelte:head>

{#snippet loadingIcon()}<Loading />{/snippet}

{#snippet pairingIcon()}<Icon icon={NotificationNew} />{/snippet}

{#snippet processingPanel()}
	{#if processing}<ProcessingPanel status={processing} timeZone={data.timeZone} />{/if}
{/snippet}

<Shell name="FleetRanger" {nav} account={data.account} theme={data.theme} {actions}>
	{@render children()}
</Shell>
