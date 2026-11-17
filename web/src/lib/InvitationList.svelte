<!--
	@component
	The person's invitations, as Carbon's contained list with a title; each
	row says who invited them where, with Accept and Decline below it
	(carbon-expert's advice: small buttons, Accept primary, Decline
	secondary; no confirmation, as declining loses no data). Both post
	straight to the backend: accepting opens that organisation, declining
	comes back to this page without it.
-->
<script lang="ts">
	import type { Invitation } from '#lib/server/backend.js';
	import Button from '#lib/ui/Button.svelte';

	interface Props {
		invitations: Invitation[];
		/** Shown above the list, e.g. beside other lists; without it the list
		 * is labelled for screen readers only. */
		title?: string;
		/** Where to go after accepting; the home page otherwise. */
		returnTo?: string;
		/** This page, to come back to after declining. */
		back: string;
	}

	let { invitations, title, returnTo, back }: Props = $props();

	const id = $props.id();
	// The answer being sent, against a second click; its button says so.
	let answering: { organisation: string; accept: boolean } | null = $state(null);

	function answer(event: SubmitEvent, organisation: string, accept: boolean) {
		if (answering) event.preventDefault();
		else answering = { organisation, accept };
	}

	function label(organisation: string, accept: boolean): string {
		const sending = answering?.organisation === organisation && answering.accept === accept;
		if (accept) return sending ? 'Accepting…' : 'Accept';
		return sending ? 'Declining…' : 'Decline';
	}
</script>

<svelte:window onpageshow={() => (answering = null)} />

<div>
	{#if title}<h2 id="{id}-title">{title}</h2>{/if}
	<ul
		aria-label={title ? undefined : 'Invitations'}
		aria-labelledby={title ? `${id}-title` : undefined}
	>
		{#each invitations as invitation (invitation.organisation_id)}
			{@const busy = answering !== null}
			<li>
				<p>
					{#if invitation.invited_by}
						{invitation.invited_by} invited you to {invitation.organisation_name}
					{:else}
						You're invited to {invitation.organisation_name}
					{/if}
				</p>
				<div class="actions">
					<form
						method="post"
						action="/auth/invitations/accept"
						onsubmit={(event) => answer(event, invitation.organisation_id, true)}
					>
						<input type="hidden" name="organisation" value={invitation.organisation_id} />
						{#if returnTo}<input type="hidden" name="return_to" value={returnTo} />{/if}
						<input type="hidden" name="back" value={back} />
						<Button type="submit" size="sm" aria-disabled={busy || undefined}>
							{label(invitation.organisation_id, true)}
						</Button>
					</form>
					<form
						method="post"
						action="/auth/invitations/decline"
						onsubmit={(event) => answer(event, invitation.organisation_id, false)}
					>
						<input type="hidden" name="organisation" value={invitation.organisation_id} />
						<input type="hidden" name="back" value={back} />
						<Button type="submit" kind="secondary" size="sm" aria-disabled={busy || undefined}>
							{label(invitation.organisation_id, false)}
						</Button>
					</form>
				</div>
			</li>
		{/each}
	</ul>
</div>

<style>
	/* Carbon's contained list title, on-page: heading-compact-01 in a 32px row. */
	h2 {
		display: flex;
		align-items: center;
		min-height: var(--cds-spacing-07);
		margin: 0;
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	ul {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-05);
		padding: var(--cds-spacing-05);
		background: var(--cds-layer-01);
	}

	li + li {
		border-top: 1px solid var(--cds-border-subtle-01);
	}

	p {
		margin: 0;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
	}

	/* Side by side, as Carbon's button sets. */
	.actions {
		display: flex;
		gap: 1px;
	}
</style>
