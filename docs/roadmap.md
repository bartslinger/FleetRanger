# Roadmap

Status: `[ ]` todo, `[~]` in progress, `[x]` done.

## Now

**Releases:** v0.4.0 and android-v0.2.1 run on the first organisation's server; the deploy dumps the database before changing the release. v0.4.0 reworked **Sessions**: the cards show a week or a month beside a map with all its tracks, each card with a **day strip** (the day with its flights and night, each flight's times on hover); **a session opens on the page itself** (the map flies to it, its facts replace the cards, its flights follow in a table, the arrows step through sessions across weeks and months with the filters kept); a track on the map opens its session, and in an open session points at its flight's row with its times. Tracks are **cyan** with a dot for takeoff and an X for landing; the street map follows the theme and the satellite or map choice is kept for the tab. Times carry no time-zone label. v0.3.0 brought areas and list filters.

**v0.5.0 prepared** (images on the server, playbook to run): **batteries** in Equipment, with a page per battery (capacity over time, its flights), linked from each flight, and fixes in the parser's battery records. The release reprocesses every log once, with the stored DJI keys (about an hour on the server). nginx now also compresses JSON. See "Batteries" below.

**Next:** numbers at takeoffs and landings, and perhaps a colour per pilot (teal 50 a candidate second colour).

**Public website** (fleetranger.org): the landing page is done and is being published on GitHub Pages; what's left is enabling Pages with the custom domain and the DNS records.

Login works locally (phase 4). Everyone is a user, with an optional team and an optional login; admins manage users, teams and equipment in the web app. Sessions are the start page; the logbook of all flights is under "Flights". Each user sees only the flights the policy allows, and the logbook sets the pilot and team of flights in batches and filters them by drone and controller. Members see and name their team's equipment; a flight's page links to its drone's and controller's pages. Each flight has a page with its track on a Mapbox map, linked from the logbook. Sessions (phase 3) are in progress: only uploaded flights join a session by themselves (per pilot by a time gap, on the same night's sync); other flights are grouped by hand from the Flights list ("Without session", New session, Add to session), where a Suggestions view groups those flights into suggested sessions to review and create one by one. The Sessions list (cards grouped by month, with a map previewing the hovered session from laptop width; or a table) leads to a page per session with all tracks on one map and its totals. Sessions can be merged and split. Both lists filter by days, pilots, drones, controllers and teams (for admins), and a session's page steps through the filtered list. The remote controller app (phase 6) has started: it runs on a DJI RC Pro Enterprise, asks who's flying (the install's pilots, kept for working offline), opens DJI Pilot 2 and uploads finished logs, only those the server lacks. Controllers pair with the install: the controller shows a code and QR code, any user enters it on /pair (announced by a notification in the header). The audit log waits until there is a working version of the app. Deploying login needs the OIDC settings, `FLEETRANGER_PUBLIC_URL` and a production redirect URI through the Ansible repo. Ingest works on 44 test logs locally; deploying it to the server and importing all logs there is on hold.

## Phase 1: backend skeleton

- [x] Repository, license, docs
- [x] axum app with config, logging and a health endpoint
- [x] PostgreSQL with migrations
- [x] Docker image, cross-built for x86_64
- [x] Compose stack and image shipping script
- [x] Ansible role in the deployment repo
- [x] Deployed to the first install, with no public route yet

## Phase 2: ingest

- [x] Vendored dji-log-parser with fixes
- [~] Import existing raw logs from disk (works locally; not yet on the server)
- [x] DJI keychain requests, recorded per log
- [x] Parse into flights: times, time in air, distance, track, drone/controller/battery serials
- [ ] Re-processing of all logs after parser changes

## Phase 3: sessions

- [x] Automatic grouping by time gap
- [x] Assigning a pilot per session (sets it on all its flights)
- [x] Splitting and merging sessions (moving a flight is splitting it off and merging)
- [x] Only uploads join sessions by themselves (same night's sync); the rest by hand: new session, add to session, flights without session
- [x] Suggested sessions to review (one by one; making all of them at once only if it turns out to be needed)

## Phase 4: login and access

- [x] OIDC login (Google), server-side sessions (not yet deployed)
- [x] Users (with optional team and login), teams, devices (managed by admins; drones and controllers registered from logs)
- [x] Policy module and access rules (flight visibility, setting a flight's pilot, moving flights)
- [ ] Audit log

## Phase 5: web

- [x] Design system foundation: tokens, fonts, UI shell, 2x Grid, first components, theme setting
- [x] Page layout: Gray 10 for light mode, a dark shell with the side nav rail, page header without the band
- [x] Logbook (paging, sorting, pilot and team, batch actions to set them)
- [x] Flight detail with map
- [x] Session overview (list, and a page with all tracks on one map)
- [x] Admin pages: users, teams, equipment
- [x] Archiving flights, with an "Archived" filter to see and restore them
- [x] Push log processing and pairing status to the browser with server-sent events, instead of polling, for a snappier interface

## Phase 6: remote controller app

- [x] App skeleton: Kotlin, Compose with our own Carbon components (light only), built in a pinned Docker image
- [x] Pilot picker with the install's pilots, kept for working offline, last pilot as the default, with "Not in the list"
- [x] Launching DJI Pilot 2
- [x] Syncing finished logs over approved Wi-Fi networks, any Wi-Fi or any connection
- [x] Uploading only the logs the server lacks (asked by hash before uploading)
- [x] Pairing: linking a (new) controller to the organisation, which any user can do, with unpairing and the download page
- [~] Onboarding asks which logs to upload (all, or from a date that starts at today, with the number of logs each covers); today's until chosen

## Public website (fleetranger.org)

- [x] Landing page: what FleetRanger does, for whom, self-hosted, source and contact
- [x] Static build reusing the web app's components
- [~] Hosting on GitHub Pages, built by a GitHub Action from main

## MCP (AI assistants)

- [x] Read-only summary tools at `/mcp`, off unless the install switches it on
- [~] OAuth, so assistants in claude.ai and others can connect: consent page and connected apps work locally; security review in progress
- [ ] `/mcp` in the nginx routes (Ansible repo)

## Areas

- [x] Schema: areas, and time and distance per flight and area
- [x] Admin page: drawing areas on the map with snapping, clipped on save
- [x] Time and distance per area for every flight, recomputed when areas change
- [x] Areas on the flight and session pages, filters in the lists, seconds per area in MCP
- [x] Redesign: one polygon per area, a list beside the map with Mapbox-style drawing tools, one Areas fact instead of a table
- [x] Carbon review of the areas UI (open: a "Discard changes?" question when cancelling a changed drawing; no keyboard alternative for drawing, see design-system.md)

## Batteries

- [x] Parser: static battery information in version 14 logs
- [x] Schema: batteries, and what each flight's log says about its battery
- [x] Equipment › Batteries, and a page per battery with its capacity over time (Carbon Charts) and flights
- [x] Carbon review of the battery UI
- [x] Parser: charge and cell records with an extra byte after the index (checked on all 3,611 logs of the first organisation)
- [x] A flight's page links its battery
- [ ] Release v0.5.0: the migration reprocesses every log once, with the stored DJI keys
- [x] Compression: scripts and styles turned out to be compressed already; nginx now also gzips JSON (Ansible)

## Before production use

- [ ] Backups: PostgreSQL dumps and raw logs
- [ ] Restrict development access to servers: a deploy user without sudo, or no access at all
- [ ] External secrets (DJI app key, OIDC client secret) only in a vault that developers can't decrypt

## Later

- Session reports with organisation-specific fields
- Sharing flights, also outside the organisation
- Invite links for accounts
- Suggesting operations from flights that overlap in time and area
- Link media

## Open questions

- Who sees flights with no pilot and no team?
