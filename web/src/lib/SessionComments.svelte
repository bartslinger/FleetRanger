<!--
	@component
	A session's comments: a record of what happened, under its facts (user's
	choice, 2026-10-25: available at first glance). Oldest first, each with
	its author and time. Editors add one with "Add comment" in the title row,
	which opens a text area at the end, so while browsing the panel shows only
	the comments (user's choice, 2026-10-25). Authors edit their own in place
	or delete them from the comment's overflow menu.

	Carbon has no comments component. carbon-expert's advice: an on-page
	contained list with a title, a low-emphasis Add that opens a text area
	with a counter in place, editing in place with Save and Cancel, and a danger confirmation
	to delete, since the record's time can't be written again.
-->
<script lang="ts">
	import { tick } from 'svelte';
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { dateTime } from '#lib/format.js';
	import Add from '@carbon/icons/es/add/16.js';
	import Button from '#lib/ui/Button.svelte';
	import InlineNotification from '#lib/ui/InlineNotification.svelte';
	import Modal from '#lib/ui/Modal.svelte';
	import OverflowMenu from '#lib/ui/OverflowMenu.svelte';
	import TextArea, { characters } from '#lib/ui/TextArea.svelte';
	import { toast } from '#lib/ui/toasts.svelte.js';
	import type { SessionComment } from '#lib/server/backend.js';

	interface Props {
		sessionId: string;
		comments: SessionComment[];
		timeZone: string;
		/** The logged-in user, who changes only their own comments. */
		userId: string;
		/** sessions.edit on the session: adding, and changing one's own. */
		canWrite: boolean;
		/** The backend's messages for a refused add, edit or delete. */
		addError?: string;
		editError?: string;
		deleteError?: string;
	}

	let { sessionId, comments, timeZone, userId, canWrite, addError, editError, deleteError }: Props =
		$props();

	/** The backend's limit, in characters. */
	const MAX_LENGTH = 4000;
	const TOO_LONG = 'Shorten the comment to 4,000 characters.';
	/** Something to save: not empty and not too long. */
	const savable = (text: string) => text.trim() !== '' && characters(text.trim()) <= MAX_LENGTH;

	/** The start of a comment, to say which one a dialog is about. */
	function excerpt(text: string) {
		const line = text.trim().split('\n')[0];
		return characters(line) > 80 ? `${[...line].slice(0, 80).join('').trimEnd()}…` : line;
	}

	// The page's form actions, keeping the page's query (the open session).
	const action = (name: string) => (page.url.search ? `${page.url.search}&/${name}` : `?/${name}`);

	const written = (comment: SessionComment) => dateTime(comment.created_at, timeZone);

	// Adding.
	let draft = $state('');
	let adding = $state(false);
	let addRefused = $state(false);
	let addField: HTMLTextAreaElement | null = $state(null);
	// The text area shows only when asked for; a draft stays until added.
	let writing = $state(false);
	let addTrigger: HTMLElement | null = $state(null);

	async function startWriting() {
		writing = true;
		addRefused = false;
		await tick();
		addField?.focus();
	}

	async function stopWriting() {
		writing = false;
		await tick();
		addTrigger?.querySelector('button')?.focus();
	}

	// Editing one comment in place.
	let editing: string | null = $state(null);
	let editDraft = $state('');
	let saving = $state(false);
	let editRefused = $state(false);
	let editField: HTMLTextAreaElement | null = $state(null);
	// Each own comment's menu button, to give focus back to.
	const triggers: Record<string, HTMLElement | null> = $state({});

	async function edit(comment: SessionComment) {
		editing = comment.id;
		editDraft = comment.body;
		editRefused = false;
		await tick();
		editField?.focus();
	}

	async function stopEditing() {
		const id = editing;
		editing = null;
		await tick();
		if (id) triggers[id]?.focus();
	}

	// Deleting, after a confirmation.
	let deleting: SessionComment | null = $state(null);
	let deleteOpen = $state(false);
	let heading: HTMLElement | null = $state(null);
</script>

<section class="comments layer-01" aria-labelledby="comments-title">
	<div class="header">
		<h2 class="title" id="comments-title" tabindex="-1" bind:this={heading}>Comments</h2>
		{#if canWrite && !writing}
			<span class="add-trigger" bind:this={addTrigger}>
				<Button kind="ghost" icon={Add} onclick={startWriting}>Add comment</Button>
			</span>
		{/if}
	</div>
	{#if comments.length === 0}
		<p class="empty">{canWrite ? 'No comments yet.' : 'No comments.'}</p>
	{:else}
		<ul class="list">
			{#each comments as comment (comment.id)}
				<li>
					{#snippet meta()}
						<p class="meta">
							<span class="author">{comment.author_name}</span>
							<span class="when">
								<time datetime={comment.created_at}>{written(comment)}</time
								>{#if comment.edited_at}<span title={dateTime(comment.edited_at, timeZone)}
										>&ensp;·&ensp;Edited</span
									>{/if}
							</span>
						</p>
					{/snippet}
					{#if editing === comment.id}
						<form
							class="edit"
							method="POST"
							action={action('editComment')}
							use:enhance={() => {
								saving = true;
								return async ({ result, update }) => {
									saving = false;
									await update({ reset: false });
									editRefused = result.type === 'failure';
									if (result.type === 'success') stopEditing();
								};
							}}
						>
							<input type="hidden" name="session_id" value={sessionId} />
							<input type="hidden" name="comment_id" value={comment.id} />
							<!-- Who wrote it and when stay in view while it changes. -->
							{@render meta()}
							{#if editRefused && editError}
								<InlineNotification kind="error" title={editError} />
							{/if}
							<TextArea
								label="Edit comment"
								name="body"
								limit={MAX_LENGTH}
								overLimit={TOO_LONG}
								bind:value={editDraft}
								bind:ref={editField}
								onkeydown={(event) => {
									if (event.key === 'Escape') {
										event.preventDefault();
										stopEditing();
									}
								}}
							/>
							<div class="buttons">
								<Button type="submit" disabled={saving || !savable(editDraft)}>Save</Button>
								<Button kind="secondary" onclick={stopEditing}>Cancel</Button>
							</div>
						</form>
					{:else}
						<div class="comment">
							{@render meta()}
							<p class="body">{comment.body}</p>
						</div>
						{#if canWrite && comment.author_id === userId}
							<span class="action">
								<OverflowMenu
									variant="row"
									vertical
									label="Actions for your comment of {written(comment)}"
									tooltip="Actions"
									bind:trigger={
										() => triggers[comment.id] ?? null,
										(element) => (triggers[comment.id] = element)
									}
									items={[
										{ label: 'Edit', onselect: () => edit(comment) },
										{
											label: 'Delete',
											danger: true,
											onselect: () => {
												deleting = comment;
												deleteOpen = true;
											}
										}
									]}
								/>
							</span>
						{/if}
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
	{#if canWrite && writing}
		<form
			class="add"
			method="POST"
			action={action('addComment')}
			use:enhance={() => {
				adding = true;
				return async ({ result, update }) => {
					adding = false;
					await update({ reset: false });
					addRefused = result.type === 'failure';
					if (result.type === 'success') {
						draft = '';
						stopWriting();
						toast('Comment added.');
					}
				};
			}}
		>
			<input type="hidden" name="session_id" value={sessionId} />
			{#if addRefused && addError}
				<InlineNotification kind="error" title={addError} />
			{/if}
			<TextArea
				label="Comment"
				name="body"
				rows={3}
				limit={MAX_LENGTH}
				overLimit={TOO_LONG}
				bind:value={draft}
				bind:ref={addField}
				onkeydown={(event) => {
					if (event.key === 'Escape') {
						event.preventDefault();
						stopWriting();
					}
				}}
			/>
			<div class="buttons">
				<Button type="submit" disabled={adding || !savable(draft)}>Add comment</Button>
				<Button kind="secondary" onclick={stopWriting}>Cancel</Button>
			</div>
		</form>
	{/if}
</section>

<Modal
	title="Delete comment"
	submitLabel="Delete"
	danger
	trigger={false}
	bind:open={deleteOpen}
	action="?/deleteComment"
	error={deleteError}
	returnFocus={() =>
		deleting && triggers[deleting.id]?.isConnected
			? (triggers[deleting.id] ?? undefined)
			: (heading ?? undefined)}
	onsuccess={() => toast('Comment deleted.')}
>
	<input type="hidden" name="session_id" value={sessionId} />
	<input type="hidden" name="comment_id" value={deleting?.id} />
	<p>
		“{deleting ? excerpt(deleting.body) : ''}”, written {deleting ? written(deleting) : ''}, is
		deleted for everyone. This can’t be undone.
	</p>
</Modal>

<style>
	.comments {
		background: var(--cds-layer-01);
	}

	/* The list title on the panel, 48px high as a list row, with Add comment
	   at its end. */
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: var(--cds-spacing-09);
		padding-left: var(--cds-spacing-05);
	}

	.title {
		margin: 0;
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.title:focus {
		outline: none;
	}

	.empty {
		margin: 0;
		padding: 0 var(--cds-spacing-05) var(--cds-spacing-05);
		color: var(--cds-text-secondary);
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
	}

	.list {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	/* Rows grow with their text; a divider between them, as in Carbon's
	   contained list. */
	li {
		display: flex;
		align-items: flex-start;
		border-top: 1px solid var(--cds-border-subtle);
	}

	.comment {
		flex: 1;
		min-width: 0;
		padding: var(--cds-spacing-04) var(--cds-spacing-05);
	}

	.meta {
		display: flex;
		flex-wrap: wrap;
		column-gap: var(--cds-spacing-03);
		margin: 0 0 var(--cds-spacing-02);
	}

	.author {
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-heading-compact-01-letter-spacing);
	}

	.when {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-heading-compact-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	/* The text as written, its line breaks kept. */
	.body {
		margin: 0;
		font-size: var(--cds-body-01-font-size);
		line-height: var(--cds-body-01-line-height);
		letter-spacing: var(--cds-body-01-letter-spacing);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	.action {
		display: flex;
		flex: none;
	}

	.edit,
	.add {
		display: grid;
		gap: var(--cds-spacing-05);
		padding: var(--cds-spacing-05);
	}

	.edit {
		flex: 1;
		min-width: 0;
	}

	/* Between the comments and the field to add one. */
	.add {
		border-top: 1px solid var(--cds-border-subtle);
	}

	.buttons {
		display: flex;
		gap: var(--cds-spacing-05);
	}
</style>
