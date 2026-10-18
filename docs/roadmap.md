# Roadmap

Status: `[ ]` todo, `[~]` in progress, `[x]` done.

## Now

Login works locally (phase 4), and admins manage pilots, teams, equipment and users in the web app. Each account sees only the flights the policy allows. Next is setting a flight's pilot and moving flights between teams, then sessions (phase 3), which are grouped per pilot. The audit log waits until there is a working version of the app. Deploying login needs the OIDC settings, `FLEETRANGER_PUBLIC_URL` and a production redirect URI through the Ansible repo. Ingest works on 44 test logs locally; deploying it to the server and importing all logs there is on hold.

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

Waits for pilots (phase 4): sessions are grouped per pilot.

- [ ] Automatic grouping by time gap
- [ ] Assigning a pilot per session
- [ ] Moving flights between sessions, splitting and merging sessions

## Phase 4: login and access

- [x] OIDC login (Google), server-side sessions (not yet deployed)
- [x] Pilots, accounts, teams, devices (managed by admins; drones and controllers registered from logs)
- [~] Policy module and access rules (flight visibility done; setting a flight's pilot and moving flights next)
- [ ] Audit log

## Phase 5: web

- [x] Design system foundation: tokens, fonts, UI shell, 2x Grid, first components, theme setting
- [x] Page layout: Gray 10 for light mode, a dark shell with the side nav rail, page header without the band
- [~] Logbook (paging and sorting done; no pilots or access rules yet)
- [ ] Flight detail with map
- [ ] Session overview
- [x] Admin pages: pilots, teams, equipment, users

## Phase 6: remote controller app

- [ ] Pilot picker that works offline, last pilot as the default
- [ ] Creating pilots on the controller
- [ ] Launching DJI Pilot 2
- [ ] Syncing finished logs over selected Wi-Fi networks
- [ ] Device enrollment

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
- Do session "time on/off" values mean first takeoff and last landing?
- Who sees flights with no pilot and no team?
