<!--
	@component
	The organisations someone can open, as Carbon's contained list with
	clickable rows (carbon-expert's advice): one form per organisation, posting
	to the same switch as the profile panel. The last used one
	is first, and marked when there is more than one.
-->
<script lang="ts">
	import ArrowRight from '@carbon/icons/es/arrow--right/16.js';
	import Icon from '#lib/ui/Icon.svelte';

	interface Props {
		organisations: { id: string; name: string; last_used: boolean }[];
		/** A visible list title, e.g. "Your organisations"; without it the list
		 * is labelled for screen readers only. */
		title?: string;
		/** Where to go after opening one; the home page otherwise. */
		returnTo?: string;
	}

	let { organisations, title, returnTo }: Props = $props();

	const id = $props.id();
	// The last used one first.
	const sorted = $derived(
		[...organisations].sort((a, b) => Number(b.last_used) - Number(a.last_used))
	);
	// The organisation being opened, against a second click.
	let opening: string | null = $state(null);
</script>

<svelte:window onpageshow={() => (opening = null)} />

<div>
	{#if title}<h2 id="{id}-title">{title}</h2>{/if}
	<ul
		aria-label={title ? undefined : 'Organisations'}
		aria-labelledby={title ? `${id}-title` : undefined}
	>
		{#each sorted as organisation (organisation.id)}
			<li>
				<form
					method="post"
					action="/auth/organisation"
					onsubmit={(event) => {
						if (opening) event.preventDefault();
						else opening = organisation.id;
					}}
				>
					<input type="hidden" name="organisation" value={organisation.id} />
					{#if returnTo}<input type="hidden" name="return_to" value={returnTo} />{/if}
					<button
						type="submit"
						aria-disabled={opening !== null && opening !== organisation.id ? true : undefined}
					>
						<span class="text">
							<span class="name">
								{opening === organisation.id ? `Opening ${organisation.name}…` : organisation.name}
							</span>
							{#if organisation.last_used && organisations.length > 1}
								<span class="note">Last used</span>
							{/if}
						</span>
						<span class="arrow"><Icon icon={ArrowRight} /></span>
					</button>
				</form>
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

	li + li {
		border-top: 1px solid var(--cds-border-subtle-01);
	}

	button {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--cds-spacing-05);
		width: 100%;
		min-height: var(--cds-size-large);
		padding: var(--cds-spacing-04) var(--cds-spacing-05);
		border: 0;
		background: var(--cds-layer-01);
		color: var(--cds-text-primary);
		text-align: left;
		cursor: pointer;
	}

	button:hover {
		background: var(--cds-layer-hover-01);
	}

	button:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	button:active {
		background: var(--cds-layer-active-01);
	}

	/* The other rows while one opens: Carbon's disabled state. */
	button[aria-disabled='true'] {
		background: var(--cds-layer-01);
		color: var(--cds-text-disabled);
		cursor: not-allowed;
	}

	button[aria-disabled='true'] .note,
	button[aria-disabled='true'] .arrow {
		color: var(--cds-text-disabled);
	}

	.text {
		display: flex;
		flex-direction: column;
	}

	.name,
	.note {
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.note {
		color: var(--cds-text-secondary);
	}

	.arrow {
		display: flex;
		color: var(--cds-icon-interactive);
	}
</style>
