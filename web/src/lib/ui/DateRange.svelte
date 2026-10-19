<!--
	@component
	Carbon's date picker, range variant, for a filter: "From" and "To" fields
	that take a typed date or one picked in a calendar, where the first click
	is the first day and the second the last (the same day twice for one
	day), as when booking a trip. Days are "2026-10-12"; `to` is null for a
	single day. Bits UI handles the fields' segments, the calendar's keyboard
	and focus. Below md the fields are the phone's own date pickers instead,
	as our selects are native there.
-->
<script lang="ts">
	import { DateRangePicker } from 'bits-ui';
	import { parseDate, type DateValue } from '@internationalized/date';
	import Calendar from '@carbon/icons/es/calendar/16.js';
	import ChevronLeft from '@carbon/icons/es/chevron--left/16.js';
	import ChevronRight from '@carbon/icons/es/chevron--right/16.js';
	import Icon from './Icon.svelte';

	interface Props {
		from: string | null;
		to: string | null;
		/** A new span: both days, or null to clear it. */
		onchange: (from: string | null, to: string | null) => void;
	}

	let { from, to, onchange }: Props = $props();

	const id = $props.id();

	type Range = { start: DateValue | undefined; end: DateValue | undefined };
	const range = (from: string | null, to: string | null): Range => ({
		start: from ? parseDate(from) : undefined,
		end: from ? parseDate(to ?? from) : undefined
	});

	// The applied span, again after each navigation; the picker's while choosing.
	let value: Range = $derived(range(from, to));

	function changed(next: Range) {
		value = next;
		if (next.start && next.end) {
			const first = next.start.toString();
			const last = next.end.toString();
			if (first !== from || (last === first ? null : last) !== to) {
				onchange(first, last === first ? null : last);
			}
		} else if (!next.start && !next.end && from) {
			onchange(null, null);
		}
	}

	// As in Carbon, the calendar opens when a field gets focus (click or Tab),
	// not only from the calendar icon. Focus within the fields keeps it open.
	let open = $state(false);
	let fields: HTMLElement | undefined = $state();
	// Closing hands focus back to the field; that focus mustn't reopen it.
	let closedAt = 0;
	function focusedField(event: FocusEvent) {
		if (open || (event.target as HTMLElement).closest('.date-trigger')) return;
		if (performance.now() - closedAt < 300) return;
		open = true;
	}

	// Phones: the native fields. A first day after the last moves the last.
	function nativeFrom(day: string) {
		if (!day) return onchange(null, null);
		onchange(day, to && to > day ? to : null);
	}
	function nativeTo(day: string) {
		if (!from) return onchange(day || null, null);
		onchange(from, day && day > from ? day : null);
	}
</script>

<div class="date-range">
	<div class="wide">
		<DateRangePicker.Root
			{value}
			onValueChange={changed}
			bind:open
			locale="en-GB"
			weekStartsOn={1}
			weekdayFormat="short"
			fixedWeeks
		>
			<div class="fields" bind:this={fields} onfocusin={focusedField}>
				{#each [['start', 'From'], ['end', 'To']] as const as [type, label] (type)}
					<div class="field">
						<span class="label" id={`${id}-${type}`}>{label}</span>
						<DateRangePicker.Input {type} class="date-input" aria-labelledby={`${id}-${type}`}>
							{#snippet children({ segments })}
								{#each segments as { part, value: text }, index (index)}
									<DateRangePicker.Segment {part} class="date-segment"
										>{text}</DateRangePicker.Segment
									>
								{/each}
							{/snippet}
						</DateRangePicker.Input>
					</div>
				{/each}
				<DateRangePicker.Trigger class="date-trigger" aria-label="Open the calendar">
					<Icon icon={Calendar} />
				</DateRangePicker.Trigger>
			</div>
			<DateRangePicker.Content
				class="date-menu"
				customAnchor={fields}
				sideOffset={0}
				align="start"
				collisionPadding={16}
				onCloseAutoFocus={() => (closedAt = performance.now())}
				onInteractOutside={(event) => {
					// The fields are part of the picker.
					if (fields?.contains(event.target as Node)) event.preventDefault();
				}}
				onFocusOutside={(event) => {
					if (fields?.contains(event.target as Node)) event.preventDefault();
				}}
			>
				<DateRangePicker.Calendar class="calendar">
					{#snippet children({ months, weekdays })}
						<DateRangePicker.Header class="calendar-header">
							<DateRangePicker.PrevButton class="calendar-nav" aria-label="Previous month">
								<Icon icon={ChevronLeft} />
							</DateRangePicker.PrevButton>
							<DateRangePicker.Heading class="calendar-heading" />
							<DateRangePicker.NextButton class="calendar-nav" aria-label="Next month">
								<Icon icon={ChevronRight} />
							</DateRangePicker.NextButton>
						</DateRangePicker.Header>
						{#each months as month (month.value)}
							<DateRangePicker.Grid class="calendar-grid">
								<DateRangePicker.GridHead>
									<DateRangePicker.GridRow class="calendar-row">
										{#each weekdays as weekday, index (index)}
											<DateRangePicker.HeadCell class="calendar-weekday">
												{weekday.slice(0, 2)}
											</DateRangePicker.HeadCell>
										{/each}
									</DateRangePicker.GridRow>
								</DateRangePicker.GridHead>
								<DateRangePicker.GridBody>
									{#each month.weeks as week, index (index)}
										<DateRangePicker.GridRow class="calendar-row">
											{#each week as date (date.toString())}
												<DateRangePicker.Cell {date} month={month.value} class="calendar-cell">
													<DateRangePicker.Day class="calendar-day" />
												</DateRangePicker.Cell>
											{/each}
										</DateRangePicker.GridRow>
									{/each}
								</DateRangePicker.GridBody>
							</DateRangePicker.Grid>
						{/each}
					{/snippet}
				</DateRangePicker.Calendar>
			</DateRangePicker.Content>
		</DateRangePicker.Root>
	</div>

	<div class="narrow fields">
		<div class="field">
			<label class="label" for={`${id}-native-from`}>From</label>
			<input
				id={`${id}-native-from`}
				type="date"
				class="native"
				value={from ?? ''}
				onchange={(event) => nativeFrom(event.currentTarget.value)}
			/>
		</div>
		<div class="field">
			<label class="label" for={`${id}-native-to`}>To</label>
			<input
				id={`${id}-native-to`}
				type="date"
				class="native"
				value={to ?? from ?? ''}
				min={from ?? undefined}
				onchange={(event) => nativeTo(event.currentTarget.value)}
			/>
		</div>
	</div>
</div>

<style>
	.fields {
		position: relative;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: var(--cds-spacing-03);
	}

	.label {
		color: var(--cds-text-secondary);
		font-size: var(--cds-label-01-font-size);
		line-height: var(--cds-label-01-line-height);
		letter-spacing: var(--cds-label-01-letter-spacing);
	}

	/* Below md the phone's own pickers, from md Carbon's calendar. */
	@media (max-width: 41.99rem) {
		.wide {
			display: none;
		}
	}

	@media (min-width: 42rem) {
		.narrow {
			display: none;
		}
	}

	/* Carbon's date input: a field, 40px, with the bottom border. */
	:global(.date-input),
	.native {
		display: flex;
		align-items: center;
		box-sizing: border-box;
		width: 100%;
		height: var(--cds-size-medium);
		padding: 0 var(--cds-spacing-05);
		border: 0;
		border-bottom: 1px solid var(--cds-border-strong);
		border-radius: 0;
		background: var(--cds-field);
		color: var(--cds-text-primary);
		font: inherit;
		font-size: var(--cds-body-compact-01-font-size);
		line-height: var(--cds-body-compact-01-line-height);
		letter-spacing: var(--cds-body-compact-01-letter-spacing);
		font-variant-numeric: tabular-nums;
	}

	/* The calendar icon in the To field's end. */
	:global(.date-input) {
		padding-right: var(--cds-spacing-09);
	}

	:global(.date-input:focus-within),
	.native:focus-visible {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	:global(.date-segment) {
		padding: 0 1px;
		outline: none;
	}

	:global(.date-segment[data-placeholder]) {
		color: var(--cds-text-placeholder);
	}

	:global(.date-segment:focus) {
		background: var(--cds-highlight);
	}

	:global(.date-trigger) {
		position: absolute;
		right: 0;
		bottom: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		width: var(--cds-size-medium);
		height: var(--cds-size-medium);
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	:global(.date-trigger:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	/* Carbon's calendar: 18rem on the layer with a shadow; 40px days. */
	:global(.date-menu) {
		z-index: 10;
		width: 18rem;
		padding: var(--cds-spacing-02) var(--cds-spacing-02) var(--cds-spacing-03);
		background: var(--cds-layer-01);
		box-shadow: 0 2px 6px 0 rgba(0, 0, 0, 0.3);
	}

	:global(.calendar-header) {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: var(--cds-size-medium);
	}

	:global(.calendar-heading) {
		color: var(--cds-text-primary);
		font-size: var(--cds-heading-compact-01-font-size);
		font-weight: var(--cds-heading-compact-01-font-weight);
		line-height: var(--cds-heading-compact-01-line-height);
	}

	:global(.calendar-nav) {
		display: flex;
		align-items: center;
		justify-content: center;
		width: var(--cds-size-medium);
		height: var(--cds-size-medium);
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--cds-icon-primary);
		cursor: pointer;
	}

	:global(.calendar-nav:hover) {
		background: var(--cds-layer-hover-01);
	}

	:global(.calendar-nav:focus-visible),
	:global(.calendar-day:focus-visible) {
		outline: 2px solid var(--cds-focus);
		outline-offset: -2px;
	}

	:global(.calendar-grid) {
		width: 100%;
		border-collapse: collapse;
	}

	:global(.calendar-weekday) {
		height: var(--cds-size-medium);
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		font-weight: 400;
	}

	:global(.calendar-cell) {
		padding: 0;
		text-align: center;
	}

	:global(.calendar-day) {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		margin: 0 auto;
		color: var(--cds-text-primary);
		font-size: var(--cds-body-compact-01-font-size);
		font-variant-numeric: tabular-nums;
		cursor: pointer;
	}

	:global(.calendar-day:hover) {
		background: var(--cds-layer-hover-01);
	}

	:global(.calendar-day[data-outside-month]) {
		color: var(--cds-text-helper);
	}

	/* Today: Carbon's 4px bar under the number. */
	:global(.calendar-day[data-today]) {
		position: relative;
		font-weight: 600;
		color: var(--cds-link-primary);
	}

	:global(.calendar-day[data-today])::after {
		content: '';
		position: absolute;
		bottom: 0.4375rem;
		left: 50%;
		width: 0.25rem;
		height: 0.25rem;
		background: var(--cds-link-primary);
		transform: translateX(-50%);
	}

	/* The span: its days highlighted, the first and last filled. */
	:global(.calendar-day[data-range-middle]),
	:global(.calendar-day[data-highlighted]) {
		background: var(--cds-highlight);
	}

	:global(.calendar-day[data-selection-start]),
	:global(.calendar-day[data-selection-end]) {
		background: var(--cds-button-primary);
		color: var(--cds-text-on-color);
	}
</style>
