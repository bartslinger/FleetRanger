# Design system

FleetRanger follows the [Carbon Design System](https://carbondesignsystem.com) as its design language: its tokens, its component and pattern guidelines, and the philosophy behind them. We build our own components and don't depend on a Carbon component library.

This document has two parts. **Our decisions** are choices made for FleetRanger. **Carbon rules** are Carbon's guidance condensed to what matters for this product, so it can be applied without rereading the whole system. Read both before UI work. For anything more detailed (exact component specifications, edge cases, a review of a page), ask the `carbon-expert` agent (`.claude/agents/carbon-expert.md`), which works from the complete Carbon documentation.

## Our decisions

### Principles

- **Laptop first.** The main use is reviewing flights, sessions and reports at a desk, so the layout and density follow Carbon's defaults for that.
- **A useful subset on mobile.** Pilots check their own sessions and fill in reports on a phone. Those screens must work at phone width, with touch-sized controls, and keep native controls (pickers, selects) on phones.
- **The same look on the remote controller.** The Android app uses the same tokens (colours, type, spacing), translated to a Jetpack Compose theme.
- **No third-party requests,** except for maps. Fonts and assets are bundled with the app. Maps are the one exception: Mapbox serves the map tiles, styles and map labels (see "Web and API" in [architecture.md](architecture.md)).

### Foundations

- **Tokens:** colour, layering, spacing, type scale and motion values come from Carbon's official packages (`@carbon/themes`, `@carbon/layout`, `@carbon/type`, Apache-2.0). `pnpm tokens` (`web/scripts/tokens.js`) exports them to CSS custom properties in `web/src/lib/ui/tokens.css`, which is committed so upgrades show as a diff. Names follow Carbon's (`--cds-layer-01`, `--cds-spacing-05`, `--cds-heading-01-font-size`). Components use only these variables, never literal values.
- **No install telemetry.** The Carbon and Plex packages run IBM's telemetry on install; `web/pnpm-workspace.yaml` denies those scripts.
- **Themes:** Carbon's Gray 10 (`g10`) for light and Gray 100 (`g100`) for dark. Gray 10 puts white containers (tables, tiles) on a light gray page. The app follows the system setting unless the user chooses light or dark in the profile panel (the header's Profile action: who is logged in, the theme as a radio group, and Log out). The choice is kept in a cookie, so the server renders it directly.
- **Shell:** a dark (Gray 100) header, side navigation and profile panel in both themes, through Carbon's inline theming (the `.theme-dark` class from `tokens.js`). From lg the side navigation is Carbon's rail: 48 px of icons, expanding to 256 px over the content on hover or keyboard focus. Below lg it is hidden behind the menu button. Every navigation item has a 16 px icon.
- **Logo** (user's choice, 2026-10-19): a shield like a ranger's arm patch, Blue 60, with a route on its side in white: a takeoff point low left, a target high right. It is the favicon (`web/src/lib/assets/favicon.svg`), the Android app's launcher icon (on white) and, at 20 px, the mark before the product name in the header. Carbon's header has no logo slot; the name stays the link's text, so the mark is decorative (`alt=""`). The name starts 8 px from the edge from lg (Carbon: 16 px) so the mark lines up with the rail's icons.
- **Page header:** the title (and later a breadcrumb) sits on the page background, without Carbon's page header band, with 32 px above and below. This deviates from Carbon, whose page header always has the band.
- **Typography:** IBM Plex Sans and IBM Plex Mono (SIL Open Font License), self-hosted from the `@ibm/plex-*` packages and bundled by Vite. Numbers in tables and statistics use tabular figures.
- **Styling:** plain CSS with custom properties, scoped per Svelte component. No utility framework.

### Components

- **Location:** `web/src/lib/ui/`, as our own code.
- **Behaviour:** interactive components (dialog, combobox, menu, tooltip, date picker) use [Bits UI](https://bits-ui.com) for keyboard handling, focus management and accessibility. Simple components (button, tag, tile, notification) are plain Svelte. Select is a styled native select, as in Carbon, because native menus work best on phones. Its open menu is the operating system's, also on desktop.
- **Icons:** from `@carbon/icons` (Apache-2.0), imported one at a time and rendered by `Icon.svelte`.
- **Spec:** each component follows the matching Carbon component's usage and style guidelines. Where we deviate, the component's file says why.
- **Rule for pages:** pages compose components and use CSS only for layout (grid, flex, gap). Colour, type and spacing decisions belong in components and tokens.
- **Known gap: drawing areas needs a pointer** (my choice, 2026-10-21). Placing, moving and removing an area's corners on the map can't be done with the keyboard, which Carbon asks of all interactive content. It is an admin-only task on a laptop; a keyboard alternative (entering corners as coordinates) is left until someone needs it. Reading the areas doesn't need the map: the list and the per-area tables carry the same information.
- **Overview:** `/design` on the dev server shows every component in its states. It isn't in the navigation (user's decision, 2026-10-19); open it by its address.

## Carbon rules

### Philosophy

- Carbon is the IBM Design Language expressed as tokens, components, patterns and guidance. Its principles: open, inclusive (accessible to everyone), modular (components combine freely), user first (based on research), and consistent (everything is designed to work together).
- Consistency is the point: it lowers cognitive load, errors and training. Spend effort on the product's own problems, not on reinventing basics.
- Style a component to fit, but never so far that it no longer looks or behaves as expected.
- Product UI is **productive**: users are focused on a job, work actively through inputs and controls, and stay a while. Success is time on task. Keep content condensed and efficient. **Expressive** moments (larger type, motion, fluid components) are rare breaks: a home page, a page header, an important moment.

### Layout and the 2x Grid

- **Mini unit:** 8 px. All sizes, margins and padding are multiples. Divide or multiply by two.
- **Breakpoints:** sm 320 px (4 columns, margin 0), md 672 (8 columns, margin 16), lg 1056 (16 columns, margin 16), xlg 1312 (16, 16), max 1584 (16, margin 24). Padding inside boxes is 16 px; the gutter is 32 px. Test at every breakpoint.
- **Style model:** product UI uses the *product and docs* model: a left navigation, content inside a maximum width (1584 px), grid aligned left. Dashboards and catalogues may use the full width.
- **Gutter modes:** type never hangs into the gutter.
  - *Wide* (default): containers stay inside the columns. Required for labelled fields (inputs, selects).
  - *Narrow* (most common in product): containers such as tables and tiles hang 16 px into the gutter, so the text inside them lines up with headings and text outside them.
  - *Condensed* (1 px): related pieces forming a whole (dashboard tiles), with 1 px `border-subtle` borders.
  - Mixing modes on one page is normal. Nested grids sit inside a column.
- **Key lines:** keep visible vertical and horizontal lines where many elements align. Continuity across pages matters more in a product than down one page: the same geometry and alignments on every screen.
- **Four columns** are the backbone of product layouts; type sits on four columns even in two- or three-column arrangements. A consistent top area (page header) anchors every page; denser content follows.
- **Grid behaviour:** to show more items, tile fixed boxes; to show more per item, use fluid columns. Header and toolbars: fluid width, fixed height. Side panels: fixed width. Data tables: fluid width.
- **Grid influencers:** opening or closing the left navigation keeps the column count; content resizes. Slide-in side panels reduce the column count.
- **Sizing scale:** 8, 16, 24, 32, 48, 64, 80 px. Box aspect ratios 1:1, 2:1, 2:3, 3:2, 4:3, 16:9.
- **Spacing tokens:** `spacing-01`…`13` = 2, 4, 8, 12, 16, 24, 32, 40, 48, 64, 80, 96, 160 px. Avoid other values. Use the grid for horizontal spacing and stacks (`gap`) between components, not component margins.
- **Space carries meaning:** nearness means relation; equal spacing means equal weight; more space around something makes it more important; white space separates groups without divider lines. Parts of a page may be dense, the whole page may not.

### Colour and layering

- Neutral grays dominate and separate zones with subtle shifts. **Blue is the action colour.** Other colours are used sparingly, for status and data.
- **Layering:** White theme: background white → `layer-01` Gray 10 → `layer-02` white → `layer-03` Gray 10. Gray 100: each layer one step lighter. A field is one layer above what it sits on (a field on `layer-01` uses `field-02` and `border-strong-02`). `layer-accent` is a supporting colour inside components (table header, unselected tabs), not a layer.
- **Tokens by role:** element + role + state (`background-hover`, `text-secondary`). Component tokens (button, tag, notification) only for their own component. Prefer contextual tokens (no number) for reusable components.
- **States:** hover is a half step and only for hover; selected one step; active two steps. Secondary text and icons turn primary on hover and when selected. Focus: 2 px `focus` border on every interactive element, with `focus-inset` where needed for 3:1 contrast. Disabled: grays, no hover or focus.
- **High contrast moments:** inverse tokens (tooltip) or inline theming, e.g. a Gray 100 shell or side panel on a White page. Only for major shifts.
- **Light and dark mode:** encouraged; follow the OS and offer a user setting. Only possible with tokens everywhere.

### Typography

- **IBM Plex**, weights Light, Regular and SemiBold. SemiBold for headings, never for long text. Italic only to emphasise words. Text colour neutral; blue only for actions and links.
- **Productive set** for product UI: `-01` body and supporting styles with fixed headings. Pair `body-compact-01` with `heading-compact-01`, and `body-01` with `heading-01`. Fluid and display styles never go inside containers.
- Sizes: `label-01` / `helper-text-01` 12 px; `body-compact-01` / `body-01` 14 px; `heading-compact-01` / `heading-01` 14 px semibold; `heading-03` 20 px (component and section headings, table and form titles); `heading-04` 28 px (page titles); `heading-05` 32 px and up for larger layout headings.
- Keep type styles consistent within a task, component or region; size signals hierarchy. A larger light heading can outrank a smaller bold one.

### Writing

- **Sentence case** for all UI text: page titles, tabs, buttons, table headers, labels. Never title case or all caps. Capitals only for names, acronyms (OK, GPS), places and the start of a sentence.
- Simple words, short sentences, present tense, active voice (passive is fine when the system is the subject). Second person ("you", "your"); "My …" only for labels of the user's own data.
- No "please" or "thank you". At most one exclamation mark, only for good news. "Can" for ability, "might" for possibility.
- Errors: what happened and how to fix it, in plain language, no codes; brief, honest, supportive. Form errors at most two lines.
- **Action labels**, used consistently: *Add* (existing thing into a context) vs *New* (start creating) vs *Create* (apply the details); *Remove* (from a context) vs *Delete* (destroy) vs *Clear* (empty fields); *Save* (keeps the view open), *Apply* (save without closing), *Cancel* (stop and close), *Close* (dismiss; never next to OK/Cancel), *Done*, *Reset*; *Filter* (narrow a list) vs *Search* (find in a set); *Edit*, *View details*, *Export*, *Import*, *Download*, *Upload*, *Back*, *Next*, *Finish*; *Log in* / *Log out* (not sign in). Combine verb and object where it helps ("Delete flight"). OK only when no specific verb fits. Never "Submit".

### Accessibility

- IBM's checklist: WCAG AA, Section 508 and EN 301 549. Contrast 4.5:1 for text, 3:1 for large text (24 px, or 19 px semibold) and for component boundaries, states and status colours.
- Never use colour alone: add an icon, shape or text.
- Semantic HTML first (`button`, `nav`, `main`, `header`), ARIA only to reinforce it. Landmarks; label repeated navigation landmarks. Source order equals visual and tab order. Heading levels match the visual hierarchy. "Skip to main content" is the first focusable element.
- Tables use `th` with `scope` and a caption.
- Every interactive element is keyboard operable with a visible focus state. Interactive icons have touch targets of at least 44 px.
- Keep information the user needs visible during a task, keep flows linear, no autoplay or flashing, support zoom and larger font settings.

### Motion

- Components have built-in micro-interaction motion; page-level motion is ours to choreograph. Default to **productive** motion: subtle and fast. **Expressive** motion only for significant moments (opening a page, a primary action, a notification).
- Easing (productive): standard `cubic-bezier(0.2, 0, 0.38, 0.9)` for elements visible throughout; entrance `cubic-bezier(0, 0, 0.38, 0.9)` for things appearing or reacting to input; exit `cubic-bezier(0.2, 0, 1, 0.9)` for things leaving for good (something that stays nearby, like a side panel, exits with standard easing).
- Durations: 70 ms (button, toggle), 110 ms (fade), 150 ms (small expansion), 240 ms (expansion, toast), 400 ms (large expansion), 700 ms (dimming). Bigger moves take longer.
- Motion moves along the grid, never diagonally. The same meaning gets the same motion. Load order: shell, then static content, then data, then the primary action, then charts. Stagger table rows by about 20 ms, under 500 ms total. Always honour reduced motion.

### Icons

- 16 px in components (20, 24 or 32 px where larger is needed), consistent across the product. Solid, one colour, 4.5:1 contrast. Icons have no states of their own; their backgrounds do. Centred with adjacent text.

### Patterns

**Navigation and the shell**
- Header: the menu button opens the left panel; the product name links only to the home page; system utilities (profile, notifications) on the right open panels rather than navigating. Product links go left, system links right.
- Use the left panel for more than five sections or when users switch often. Organise it by user tasks, never by organisation or technology. Never list unbounded content (pilots, flights) in it; use pages that drill down.
- Drilling into an object opens a page focused on it, with a breadcrumb above the title.
- **Stepping through a list from an item's page** (user's request, 2026-10-19; flights for now): "Previous flight", "3 of 47" and "Next flight" in the page header's actions, as ghost icon buttons (40 px, 48 px on phones) with tooltips. The list is the one the user came from (the breadcrumb trail's last step), with its filters and order across all its pages; without one there's no stepper. The tooltips say which way in time the list goes ("Next flight (older)"). At the ends a button is disabled in place. A step replaces the history entry, so Back returns to the list, and keeps focus on the button. No keyboard shortcuts: the map uses the arrow keys. Carbon has no stepper; this follows its pagination buttons (carbon-expert's advice).
- **Items without a name** (sessions, flights) are titled by their kind and date: "Session Mon 12 Oct 2026", "Flight 12 Oct 2026, 18:53". The description below adds what tells them apart; where the facts show beside the content only from md, the phone's description carries the most important of them (a session: times, pilot and team; wider: times, flights and air time). Dates use non-breaking spaces, so a title never wraps inside one.
- **Breadcrumbs are path-based** (Carbon's second type; user's decision, 2026-10-19): they show the pages the user went through since the last starting point (side navigation, product name), not a fixed hierarchy, as users go several levels deep (a list, a flight, its drone, that drone's flights). A step goes back to its page through the browser's history, so a filtered, paged or scrolled list is as the user left it; with a modifier key it opens as a link. Consecutive pages of the same list are one step; loops stay. A page opened directly (bookmark, new tab) shows its fixed parent instead. Steps are short names that say what they are ("Session Mon 12 Oct", "Flight 12 Oct, 18:53", "Flights · Drone-2" for a filtered list), not the full page titles (user's choice).
- Keep page state (filters, sort, page) in the URL, so users come back to where they were.
- The header shows where the user is and who is logged in.

**Empty states**
- They replace the element that would show: an empty table loses its headers and footer.
- A short, positive title ("Start by …", or "No … yet"), a body that says what will appear and how to get there, and optionally one primary action and a secondary link. Left-aligned. Never a dead end.
- Kinds: no data yet (first use), no results (suggest changing the search or filters), and errors (no permission → how to get access; system problem; configuration needed).

**Loading**
- Skeleton states only for data containers (tables, tiles, lists) and only for seconds; never for buttons, inputs, menus, modals or toasts.
- Inline loading for one component; full-screen loading only when the whole app is busy. Use a notification for anything taking minutes. Tell screen readers when something is loading, busy or has failed.

**Notifications**
- They must be relevant, timely and informative, and as undisruptive as possible.
- Statuses: info (blue), success (green), warning (yellow), error (red), each with its filled icon.
- Inline notifications sit near what they concern, stay under two lines and persist until resolved. Toasts are for system events: top right, auto-dismiss only without actions. Banners are for system-wide messages. Modals only for critical messages.
- Low contrast for supplemental messages, high contrast for urgent ones. Never dismiss a critical message on a timer.

**Status indicators**
- An icon, shape and colour plus a text label (at least three of symbol, shape, colour, text). Use plain text when no action is needed. More than five or six indicators on a page is too many. A combined status takes the most severe colour.
- Colours: red error, orange serious warning, yellow warning, green success or normal, blue information or in progress, gray draft or not started, purple undefined.

**Disabled, read-only and hidden**
- Disabled: temporarily unavailable; stays visible.
- Read-only: allowed to see but not change; readable and focusable.
- Hidden: no permission, so the action isn't shown at all (someone who can't add members never sees "Add member").

**Forms**
- Ask only what's needed, in a logical order, grouped under section headings. Labels sit on top, 1–3 words, no colon.
- Mark the minority: "(optional)" if most fields are required, "(required)" if most are optional.
- Helper text holds need-to-know information. A placeholder is only a format hint. Tooltips (the "i" icon) are never essential.
- Sensible defaults; focus the first field. More than five options → a select or combobox; radios get a default.
- In-page forms: buttons left-aligned, primary first. Dialogs and side panels: full-width buttons, primary on the right. Never pin buttons at the top.
- Validate on blur, explaining how to fix; server errors get an inline notification plus field errors. Disable the primary button while submitting.
- Fields are 40 px high, 32 px apart on pages (16–24 px in panels), 48 px before the buttons. Single column, except 2–3 related fields on one line.
- Dialog forms for fewer than five inputs; side panels for more, or when the page must stay visible.

**Dialogs**
- Short, user-triggered tasks only, used sparingly. Never nested, never a full page, no complex tables.
- A title and buttons naming the action ("Delete flight"). Cancel left, one primary right, full bleed at the bottom.
- Focus goes to the first input and is trapped; it returns to the trigger on close. Esc closes.
- Destructive or irreversible actions use a danger modal. High-impact deletes ask the user to type the name.

**Common actions**
- Delete by impact: trivial → immediate; hard to undo → confirm with consequences; expensive → danger modal plus typing the name. Afterwards, return to the list with a success notification.
- Cancel is a secondary button or link; Close is an "x" icon at the top right; Clear is an "x" inside the field.

**Disclosures**
- Popovers and menus open on click, one at a time, are not nested, are at most six columns wide, and never hide critical information. Table settings and filter menus use Reset/Apply when filtering isn't instant.

**Filtering and search**
- Filters: single select, multiselect, or categories (shown on the page, never inside a menu); batch "Apply" for slow data or many choices, instant otherwise. Show the number of active filters, with Clear and Clear all.
- The Flights filters (user's decisions, 2026-10-19): a date range first (Carbon's range date picker: From and To fields to type in, one calendar where the first click is the first day and the second the last, the same day twice for one day; on phones the native date fields), then Drones and Controllers as multiselects (MultiSelect.svelte: applied when the menu closes, a count tag that clears them, chosen ones first), for admins Teams (with "No team (queue)"), and Session. From lg the filters form a row of fixed widths that wraps rather than squeezes. A day is a day in the organisation's time zone. The Drones and Controllers lists offer only what still has flights with the other filters (not their own, so more can be added), hide the rest rather than disabling it, and always keep what was chosen (carbon-expert's advice; Carbon itself is silent on this). All in the URL (`from`, `to`, comma-separated `drone` and `controller`). Keep the row small: when more filters come, they move behind a toolbar "Filter" button with the active ones as tags (and on phones a "Filters (n)" button).
- Search fields have no label, only an icon and a specific placeholder. Always show the number of results.

**Overflow**
- Truncate long values with an ellipsis and a tooltip showing the full text. Use middle truncation for identifiers such as serial numbers. Never truncate titles, labels or messages.

**Login**
- "Log in", never "sign in". Never reveal whether an account exists. After an inactivity logout, return to the last page.

### Components (summary)

- **Data table:** a title (`heading-03`) and description are optional and sit in the table's own container, padded 16 px with 24 px below. Leave them out when the table is the page's only subject and the page title already names it; then label the table for screen readers. Rows are md (40 px), which Carbon's guidelines call the default and most common size (Carbon React's code defaults to lg, 48 px); lg and sm (32 px) are options. The header row matches the row size. Keep row hover on. Pagination goes directly below, at the same height. Sorting has three states, and the arrow shows only on the sorted column. The toolbar holds at most five global actions. Narrow gutter mode, so the title aligns with page text. Search sits in the toolbar.
- **Pagination:** items per page, "1–50 of 3,608 items", the page selector, and previous/next. On phones, only the range and previous/next remain.
- **Page header:** Carbon's page header is a full-width band (`layer-01`, bottom border `border-subtle-01`) with 24 px vertical padding and the title in `heading-04`, at most 640 px wide. It can also hold a breadcrumb, a subtitle (`heading-03`), a description (`body-01`) and page actions on the right.
- For every other component, ask `carbon-expert`.

## Status

- Done:
  - tokens, fonts and base styles
  - the UI shell, with a skip link and keyboard focus handling; dark, with the side nav rail
  - the 2x Grid (`Grid`, `Column`; narrow mode for containers)
  - components: button, tag, inline notification, toast notification, text input, select, checkbox, radio button group, data table (sortable, with toolbar and row links), pagination, empty state, page header (with breadcrumb and status), modal, tabs
  - the admin pattern (Users, Teams, Equipment): a list with a toolbar ("Show archived", "New …" in a modal), and a page per item with read-only facts, an in-page form, archiving after confirmation, and deleting after a danger confirmation where allowed. Confirmations of completed actions are toasts; errors stay inline. Save and Cancel return to the page the item was opened from as the browser's Back button does, so Back afterwards continues to the page before it (a list keeps its place); an item opened directly is replaced by its list.
  - batch actions in the logbook: select rows, then act on them from the blue bar over the toolbar. At most two actions as buttons (on phones one), the rest in an overflow menu. The header checkbox selects the page (Carbon); a fully selected page offers "Select all 340" in the bar (on phones on a line under it), after which all matching flights are selected across pages (carbon-expert's advice; Carbon doesn't cover it). Unticking a row goes back to the page's rows; another filter clears the selection. Dialogs give the number, and the server checks that number still matches before acting on all of them.
  - a track map (Mapbox, with a content switcher for satellite or map and a legend) and the flight page: the map beside read-only facts. The session page follows it, with all tracks on one map and its flights in a table below
  - views of the same data, switched with a content switcher in the page header: sessions as cards (the default) or a table. The table pages through all sessions with the date range. The cards show one week or month (user's decisions, 2026-10-21: this month by default, weeks from Monday, no date range in the cards), stepped with `PeriodNav`: a low-contrast Week | Month switcher, previous and next as ghost icon buttons around the period's name, and "This week"/"This month" while another period shows; switching unit keeps the moment (the week's Monday's month; a month's current week, else the week with its 1st). Carbon has no period navigator; it's built from its parts (carbon-expert's advice). One controls row, all 40px: the period at the start, the count and Filters at the end, on the content's edges; on phones the unit and Filters share a line, the steps go under it and the count is left out. Month headings only where a week spans two months. The cards are clickable tiles that open the session, with Carbon's arrow at the bottom right (user's decision, 2026-10-21, after leaving it out before: with hover bringing out a session on the map, a click opening it was unexpected without it); 6 of 16 columns up to xlg, 5 from there. From lg a map beside them has all the period's tracks at once (coarse, from the list: nothing loads on hover); pointing at a card brings out its session's tracks and dims the rest, clicking a track marks its card. The map fills the window from where it starts (`calc(100dvh - 12.75rem - 1rem)`: what's above it is 12.75rem at every width and font size). Below lg there's no map, only the list
  - a day strip on each session card (`DayStrip`; Carbon has no chart for this, reviewed by the Carbon expert): an outline without fill (a fill vanished into a hovered or marked tile's grey), flights as marks in `icon-secondary` (blue drew too much attention; user's feedback), night as a 10% `text-primary` tint over whatever is behind it, and notches at 06, 12 and 18 only outside the outline (user's choice after trying them across and below it). No legend or labels. Hovering a flight shows its times and air time in a tooltip with Carbon's caret, pointing at the flight; each flight's hover area is spacing-02 wider on both sides, split halfway where neighbours are closer, so the nearest flight wins (user's rule). Nothing in the strip is focusable, as it sits in the tile's link: the tooltip is for the mouse, the strip is hidden from screen readers, and the session's page lists every flight's times
  - one tooltip component (`TooltipContent`, Bits UI): inverse, body-compact-01, 16px inside the window, always with Carbon's caret. Two kinds, from Carbon's Sass: icon-only buttons (padding 2px × 16px, caret 8 × 4px, 8px off the button, shown below) and the default (padding 4px × 16px, caret 12 × 6px, 10px off; the day strip, above). The header's global actions use it too, in a portal, so their tooltips take the page's theme rather than the dark header's: Carbon's tooltip is the inverse of the surface it lands on, and in the header's theme it was a light box on the light page (carbon-expert's advice; user's approval, 2026-10-21). No tooltip while an action's panel is open
  - header actions beside Profile, each opening a panel (Carbon's header utilities don't navigate): controllers waiting to be paired, with a link to the pairing page; and for admins log processing. While logs are processed it shows Carbon's small loading indicator, and the panel a small progress bar, the current log and the latest results. A finished batch keeps the action until its panel has been seen: a checkmark, or a warning when logs failed (the bar stays green: the batch completed; error is for a batch that stopped). One without failures goes by itself after 20 s. Screen readers are told when a batch finishes. Reviewed by the Carbon expert, as are the shell, the page header and the theme
  - not yet reviewed by the Carbon expert: the rest since the dark shell, in packages (admin pages reviewed, findings pending)
  - the logbook, reviewed against Carbon
  - a theme setting (system, light or dark)
  - the login page (Carbon's login pattern for a single sign-on provider: the header shows only the product name) and the profile panel
- Known gaps against the rules above:
  - motion values (durations, easing) aren't exported as tokens yet; components write Carbon's values directly
