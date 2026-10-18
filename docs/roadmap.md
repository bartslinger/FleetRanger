# Roadmap

Status: `[ ]` todo, `[~]` in progress, `[x]` done.

## Now

Login works locally (phase 4). Everyone is a user, with an optional team and an optional login; admins manage users, teams and equipment in the web app. Each user sees only the flights the policy allows, and the logbook sets the pilot and team of flights in batches and filters them by drone and controller. Members see and name their team's equipment; a flight's page links to its drone's and controller's pages. Each flight has a page with its track on a Mapbox map, linked from the logbook. Sessions (phase 3) are in progress: flights are grouped per pilot by a time gap, and the Sessions list (a table, or cards beside a map) leads to a page per session with all tracks on one map and its totals. Sessions can be merged and split. The remote controller app (phase 6) has started: it runs on a DJI RC Pro Enterprise, asks who's flying (test names for now) and opens DJI Pilot 2. Controllers pair with the install: the controller shows a code and QR code, any user enters it on /pair (announced by a notification in the header). The audit log waits until there is a working version of the app. Deploying login needs the OIDC settings, `FLEETRANGER_PUBLIC_URL` and a production redirect URI through the Ansible repo. Ingest works on 44 test logs locally; deploying it to the server and importing all logs there is on hold.

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
- [ ] Assigning a pilot per session
- [x] Splitting and merging sessions (moving a flight is splitting it off and merging)

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

## Phase 6: remote controller app

- [x] App skeleton: Kotlin, Compose with our own Carbon components (light only), built in a pinned Docker image
- [~] Pilot picker that works offline, last pilot as the default, with "Not in the list" (with test names until pairing)
- [x] Launching DJI Pilot 2
- [ ] Syncing finished logs over approved Wi-Fi networks, any Wi-Fi or any connection
- [x] Pairing: linking a (new) controller to the organisation, which any user can do, with unpairing and the download page

## Before production use

- [ ] Backups: PostgreSQL dumps and raw logs
- [ ] Restrict development access to servers: a deploy user without sudo, or no access at all
- [ ] External secrets (DJI app key, OIDC client secret) only in a vault that developers can't decrypt

## Later

- Operations
- Session reports with organisation-specific fields
- Areas
- Sharing flights, also outside the organisation
- Invite links for accounts
- Suggesting operations from flights that overlap in time and area

## Open questions

- Re-uploads with the same file name but different content (a log synced while still being written): how should they replace the earlier version? Decide in phase 6; until then files are only deduplicated by hash.
- Who sees flights with no pilot and no team?
