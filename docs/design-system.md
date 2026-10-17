# Design system

FleetRanger follows the [Carbon Design System](https://carbondesignsystem.com) as its design language. We use Carbon's official tokens and its component and pattern guidelines, but build our own components. We don't depend on a Carbon component library.

## Principles

- **Laptop first.** The main use is reviewing flights, sessions and reports at a desk, so the layout and density follow Carbon's defaults for that.
- **A useful subset on mobile.** Pilots check their own sessions and fill in reports on a phone. Those screens must work at phone width, with touch-sized controls.
- **The same look on the remote controller.** The Android app uses the same tokens (colours, type, spacing), translated to a Jetpack Compose theme.
- **No third-party requests.** Fonts and assets are bundled with the app. The browser never contacts an outside host.

## Foundations

- **Tokens:** colour, layering, spacing, type scale and motion values come from Carbon's official packages (`@carbon/themes`, `@carbon/layout`, `@carbon/type`, Apache-2.0). `pnpm tokens` (`web/scripts/tokens.js`) exports them to CSS custom properties in `web/src/lib/ui/tokens.css`, which is committed so upgrades show as a diff. Names follow Carbon's (`--cds-layer-01`, `--cds-spacing-05`, `--cds-heading-01-font-size`). Components use only these variables, never literal values.
- **No install telemetry.** The Carbon and Plex packages run IBM's telemetry on install; `web/pnpm-workspace.yaml` denies those scripts.
- **Themes:** Carbon's White (`white`) for light and Gray 100 (`g100`) for dark. The app follows the system setting.
- **Typography:** IBM Plex Sans and IBM Plex Mono (SIL Open Font License), self-hosted from the `@ibm/plex-*` packages and bundled by Vite. Numbers in tables and statistics use tabular figures.
- **Styling:** plain CSS with custom properties, scoped per Svelte component. No utility framework.

## Components

- **Location:** `web/src/lib/ui/`, as our own code.
- **Behaviour:** interactive components (dialog, combobox, menu, tooltip, date picker) use [Bits UI](https://bits-ui.com) for keyboard handling, focus management and accessibility. Simple components (button, tag, tile, notification) are plain Svelte. Select is a styled native select, as in Carbon, because native menus work best on phones. Its open menu is the operating system's, also on desktop.
- **Icons:** from `@carbon/icons` (Apache-2.0), imported one at a time and rendered by `Icon.svelte`.
- **Overview:** `/design` on the dev server shows every component in its states.
- **Spec:** each component follows the matching Carbon component's usage and style guidelines. Where we deviate, the component's file says why.
- **Rule for pages:** pages compose components and use CSS only for layout (grid, flex, gap). Colour, type and spacing decisions belong in components and tokens.

## Status

- Exploration: three directions were compared, and the Carbon-like direction was chosen.
- Done: tokens, fonts, UI shell, and the first components (button, tag, inline notification, text input, select, data table, page header).
- Next: components as pages need them, starting with the logbook.
