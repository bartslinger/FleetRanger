# Roadmap

Status: `[ ]` todo, `[~]` in progress, `[x]` done.

## Now

Phase 1: backend skeleton.

## Phase 1: backend skeleton

- [x] Repository, license, docs
- [ ] axum app with config, logging and a health endpoint
- [ ] PostgreSQL with migrations
- [ ] Docker image, cross-built for x86_64
- [ ] Compose stack and an Ansible role in the deployment repo
- [ ] Deployed to the first install, with no public route yet

## Phase 2: ingest

- [ ] Import existing raw logs from disk
- [ ] DJI keychain requests, recorded per log
- [ ] Parse into flights: times, time in air, distance, track, drone/controller/battery serials
- [ ] Re-processing of all logs after parser changes

## Phase 3: sessions

- [ ] Automatic grouping by time gap
- [ ] Assigning a pilot per session
- [ ] Moving flights between sessions, splitting and merging sessions

## Phase 4: login and access

- [ ] OIDC login (Google), server-side sessions
- [ ] Pilots, accounts, teams, devices
- [ ] Policy module and access rules
- [ ] Audit log

## Phase 5: web

- [ ] Logbook
- [ ] Flight detail with map
- [ ] Session overview
- [ ] Admin pages: pilots, teams, devices

## Phase 6: remote controller app

- [ ] Pilot picker that works offline, last pilot as the default
- [ ] Creating pilots on the controller
- [ ] Launching DJI Pilot 2
- [ ] Syncing finished logs over selected Wi-Fi networks
- [ ] Device enrollment

## Later

- Operations
- Session reports with organisation-specific fields
- Areas
- Sharing flights, also outside the organisation
- Invite links for accounts
- Suggesting operations from flights that overlap in time and area

## Open questions

- Re-uploads with the same file name but different content (a log synced while still being written): keep them as versions, or name files by hash?
- Session time-gap threshold. The proposal is 2 hours, configurable.
- Do session "time on/off" values mean first takeoff and last landing?
- Who sees flights with no pilot and no team?
