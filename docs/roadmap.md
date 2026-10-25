# Roadmap

Status: `[ ]` todo, `[~]` in progress, `[x]` done.

## Now

**Releases:** v0.9.0 runs on the first organisation's server (2026-10-25), with android-v0.4.0; the deploy dumps the database before changing the release. v0.9.0 brings **comments on sessions** (a record of what happened, written by editors; migration 0021), deleting a session explicitly, and **areas shown as the air time in each** instead of a share. v0.8.0 and v0.8.1 brought impersonation for admins and a fix for deleting cookies over HTTPS. v0.7.0 brought **groups and roles** (see below): migrations 0018–0020 turned the teams into groups with their members' roles and dropped teams, and every flight, area and user came through; the app's group step talks to the versioned controller API, while android-v0.3.0 keeps working against it (checked on a controller before and after the switch). v0.6.0 reversed **pairing**: a user creates a code in the web app ("Pair a controller", valid 1 hour to 7 days, revocable) and the pilot types it into the controller, one box per character, so nobody logs in in the field; nginx limits pairing attempts per address. See "Clients" in [architecture.md](architecture.md). v0.5.0 brought **batteries** in Equipment, with a page per battery (capacity over time, its flights), linked from each flight, and fixes in the parser's battery records; it reprocessed all 3,611 logs without failures, and nginx now also compresses JSON. See "Batteries" below. v0.4.0 reworked **Sessions**: the cards show a week or a month beside a map with all its tracks, each card with a **day strip** (the day with its flights and night, each flight's times on hover); **a session opens on the page itself** (the map flies to it, its facts replace the cards, its flights follow in a table, the arrows step through sessions across weeks and months with the filters kept); a track on the map opens its session, and in an open session points at its flight's row with its times. Tracks are **cyan** with a dot for takeoff and an X for landing; the street map follows the theme and the satellite or map choice is kept for the tab. Times carry no time-zone label. v0.3.0 brought areas and list filters.

**Groups and roles, before multi-tenancy** (user's decision, 2026-10-23): groups replaced teams, and roles as building blocks (Viewer, Pilot, Editor, Admin) replaced the three fixed roles; see "Groups and roles" in [architecture.md](architecture.md) and the packages below. Teams, the fixed roles and holders are dropped (migration 0020); released as v0.7.0 and android-v0.4.0 on 2026-10-24, with fine-tuning after the release (user, 2026-10-24). Groups replace teams in the code, every access check asks for a permission on the groups tables, a flight's pilot and group are independent, uploads without a chosen group go to the queue, a session's own group decides who changes it, only flights without a session are archived, equipment is also seen through its flights with figures over all of them, pairing puts a controller in a group the creator chooses, holders are gone from the app, and the admin pages manage groups, their members' roles and roles on the whole organisation. The web app says "group" everywhere and shows what the user's permissions allow: sessions get a group chosen by their maker and can be moved with or without their flights, pilot lists put the group's own pilots first; MCP has the user's own access. The controller app has moved over (package 8): each pilot once, then the group they fly for, with "Other group"; it talks to a versioned controller API (`/api/controller/v1`, user's decision), while android-v0.3.0 keeps working, its uploads taking the group (team) chosen with the pilot, and it stays in landscape. Serving several organisations from one install follows after.

**Now:** releasing v0.9.0.

**Next:** numbers at takeoffs and landings, and perhaps a colour per pilot (teal 50 a candidate second colour).

**Public website** (fleetranger.org): the landing page is done and is being published on GitHub Pages; what's left is enabling Pages with the custom domain and the DNS records.

Everyone is a user, with roles in groups or on the whole organisation and an optional login; admins manage users, groups and equipment in the web app. Sessions are the start page; the logbook of all flights is under "Flights", which sets the pilot, group and session of flights in batches. Uploaded flights join a session by themselves (per pilot and group, on the same night's sync); the rest are grouped by hand or from suggested sessions to review. The remote controller app runs on a DJI RC Pro Enterprise: it pairs with a code from the web app, asks who's flying and for which group, opens DJI Pilot 2 and uploads the finished logs the server lacks. The audit log waits until there is a working version of the app.

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
- [x] Comments on sessions, and deleting a session explicitly
- [ ] Rethink the open session's layout (user, 2026-10-25): long comments under the facts push the flights table down, and pointing at a flight in the table, which brings it out on the map, happens with the map scrolled out of view

## Phase 4: login and access

- [x] OIDC login (Google), server-side sessions (not yet deployed)
- [x] Users (with roles and an optional login), groups, devices (managed by admins; drones and controllers registered from logs)
- [x] Policy module and access rules (flight visibility, setting a flight's pilot, moving flights)
- [x] Impersonation: an admin acts as another user, for support, with a banner on every page
- [ ] Audit log

## Phase 5: web

- [x] Design system foundation: tokens, fonts, UI shell, 2x Grid, first components, theme setting
- [x] Page layout: Gray 10 for light mode, a dark shell with the side nav rail, page header without the band
- [x] Logbook (paging, sorting, pilot and group, batch actions to set them)
- [x] Flight detail with map
- [x] Session overview (list, and a page with all tracks on one map)
- [x] Admin pages: users, groups, equipment
- [x] Archiving flights, with an "Archived" filter to see and restore them
- [x] Push log processing and pairing status to the browser with server-sent events, instead of polling, for a snappier interface

## Phase 6: remote controller app

- [x] App skeleton: Kotlin, Compose with our own Carbon components (light only), built in a pinned Docker image
- [x] Pilot picker with the install's pilots, kept for working offline, last pilot as the default, with "Not in the list"
- [x] Launching DJI Pilot 2
- [x] Syncing finished logs over approved Wi-Fi networks, any Wi-Fi or any connection
- [x] Uploading only the logs the server lacks (asked by hash before uploading)
- [x] Pairing: linking a (new) controller to the organisation, for whoever may pair in a group, with unpairing and the download page
- [x] Pairing with a code created in the web app and typed into the controller, replacing the approval flow
- [~] Onboarding asks which logs to upload (all, or from a date that starts at today, with the number of logs each covers); today's until chosen
- [x] Stays in landscape when the controller is turned

## Groups and roles

Each package leaves a working app with its tests passing. Groups are added beside teams first and filled from them; the code moves over package by package, and teams and holders are dropped only at the end. Each package also removes the rules in architecture.md it replaces.

- [x] 1. Docs: the decisions in architecture.md, these packages
- [x] 2. Schema proposal and first migration: groups, memberships with roles, organisation-wide roles and a group on flights, sessions, drones and controllers, filled from teams (the final SQL approved by the user)
- [x] 3. Permission checks: permissions and built-in roles in code, the check and the list filter, on the new tables, with today's behaviour (the access tests unchanged)
- [x] 4. Flights and sessions: pilot and group independent, the queue, a session's owner and the checks on every session that changes
- [x] 5. Equipment: seen through its flights, figures over all flights, pairing into a group, holders out
- [x] 6. Web admin pages: groups with their members and roles, organisation-wide roles (Carbon expert before and after)
- [x] 7. Web app elsewhere, together with MCP (user's choice, 2026-10-23): groups instead of teams, pilot lists, a session's owner, moving flights and sessions; MCP with the user's own access, read only
- [x] 8. Controller app: the group after the pilot, with "Other group"; older apps' flights go to the queue; the controller API versioned (v1)
- [x] The organisation as the top group in the UI (user's decisions, 2026-10-23; see "Groups" in architecture.md): Users as its member list, first in Groups, first on a user's page, "Organisation" for what has no group
- [x] 10. Final migration: teams, `users.role` and holders dropped (migration 0020; the server has no all-flights user, so the fill gives nobody Editor by that rule)
- [x] Release v0.7.0 and android-v0.4.0 (2026-10-24): server and app together, with the UI changes the move from teams to groups needs; fine-tuning waits until after it (user, 2026-10-24). The update from android-v0.3.0 was tested with the release candidate through dev.fleetranger.org
- [ ] After the release, UI fine-tuning (user, 2026-10-24) left from the Carbon review of the admin pages: Delete and Archive on phones, the "Show archived" toggle in other lists; the drone, controller and battery pages edit their own fields in a dialog from the page header, as Users and Groups do. From the review of the web app's groups: the phone batch bar cuts off its count ("2 flights sel…", while Carbon says the bar always shows it), and the carbon-expert's own suggestions (archive last in the flights' overflow menu after a divider, a select or a pencil for the group on the suggestion page)
- [ ] After the release, controller app tuning (user, 2026-10-23), the carbon-expert's own suggestions: make the group on the pilot tile stand out more, the wide gutter in the lists
- [ ] Remove the unversioned controller endpoints (and android-v0.3.0's pilot list in `pairing::db`) once Equipment shows no controller with android-v0.3.0

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
- [x] Release v0.5.0: the migration reprocessed every log once, with the stored DJI keys
- [x] Compression: scripts and styles turned out to be compressed already; nginx now also gzips JSON (Ansible)

## Before production use

- [ ] Backups: PostgreSQL dumps and raw logs
- [ ] Restrict development access to servers: a deploy user without sudo, or no access at all
- [ ] External secrets (DJI app key, OIDC client secret) only in a vault that developers can't decrypt

## Later

- Session reports with organisation-specific fields
- Sharing flights, also outside the organisation
- Invite links for accounts
- A stable public API (user's wish, 2026-10-24): the web app and backend still change together, but external parties should eventually talk to the backend directly. That needs a versioned API with a documented contract kept stable across releases, as the controller API already is (`/api/controller/v1`), and access for other apps (OAuth already lets apps act for a user).
- Link media

## Open questions

None at the moment.
