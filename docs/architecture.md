# Architecture decisions

The decisions made so far, each with a short reason. Open questions are listed in [roadmap.md](roadmap.md).

## Deployment

- **One install per organisation.** Flight data is sensitive, so each organisation runs its own server. Several installs may share one DJI app key.
- **Docker Compose** with three containers: `backend` (Rust), `web` (SvelteKit), `postgres`. TLS and certificates are handled by nginx on the host, which proxies to containers bound to `127.0.0.1`.
- **Images are built on a developer machine** and loaded onto the server. Servers can be small, and compiling Rust on them is impractical. Images target x86_64 and are cross-compiled rather than emulated.
- **Remote controllers sync over the internet** to the install's public HTTPS endpoint.

## Data

- **PostgreSQL**, without PostGIS for now. Flight tracks are stored as GeoJSON LineStrings in `jsonb`, in their own table so flight listings stay light. Moving to PostGIS later is a single backfill with `ST_GeomFromGeoJSON`.
- **Raw logs are files on disk**, never modified after upload. The database records their SHA-256 hash and origin. Re-uploads with the same name but different content are kept as separate versions, not overwritten.
- **Raw logs and full telemetry are not exposed** to any user, admins included. They exist so logs can be re-processed when the parser improves. Users only see derived data (times, distance, track, statistics).
- **Every DJI keychain request is recorded** per log: the exact request sent (API key redacted), the time, the response status and the keys returned.

## Identity and access

- **Login via OpenID Connect**, with Google first. Sessions are stored on the server, so they can be revoked immediately and audit records reference them.
- **Pilots and accounts are separate.** A pilot can be created on a remote controller, offline too, with an ID generated on the controller. Admins can also create pilots ahead of time. Admins link a login account to a pilot; invite links come later. Admins can merge duplicate pilots.
- **Teams.** Each pilot is in exactly one team. Membership is still a separate table, with a uniqueness constraint, so multiple teams remain possible later.
- **Flights** have a team and an optional pilot. The team is the pilot's team at flight time. A flight with no pilot takes the remote controller's team, or else goes to an organisation-level queue. Changing a pilot's team does not move their past flights.
- **Drones and remote controllers** belong to the organisation. Each can have a team and a current holder. The holder is informational only and grants no access.
- **Who sees a flight:**
  - its pilot
  - members of its team
  - participants of an operation containing it
  - the all-flights role
  - admins
  - later, people it has been shared with
- **Within a team**, any member can change a flight's pilot to another member, or to nobody. No acceptance is needed. "This wasn't me" means setting the pilot to nobody, and the flight stays in the team.
- **Admins** manage users, pilots, teams and devices, and can move flights between teams.
- **All access decisions go through one policy module** in the backend, never through ad-hoc checks in handlers.
- **Audit log:** append-only, recording every view, change, reassignment, move and merge, with the user, session, time and IP. There is no hash chain.

## Sessions and operations

- **A session is one pilot's consecutive flights**, for example a morning shift. A remote controller often powers off during a session while the pilot relocates, so power cycles are not boundaries. Sessions are grouped automatically by a time-gap rule. Users can move flights between sessions, and split or merge sessions.
- **Selecting a pilot on the remote controller** sets who is flying from that moment on.
- **An operation groups flights across teams**, for example a joint operation. Anyone can create one. Anyone with a flight in an operation sees all of its flights. Flights can only be added by people who can already see them, and must overlap the operation's time window. One operation per flight for now, through a link table, so this can be relaxed later.

## Clients

- **Backend:** Rust with axum.
- **Web:** SvelteKit.
- **Remote controller app:** Kotlin, for DJI Enterprise controllers running DJI Pilot 2. The controller is configured to launch FleetRanger on boot. After a pilot is selected (default: the last pilot), the app starts DJI Pilot 2. Finished logs sync only over Wi-Fi networks chosen in the app.

## Customer-specific needs

FleetRanger stays generic. Needs specific to one organisation are left out until they are actually built, and are then implemented in the simplest form. They are generalised only when a second organisation needs them. No customer data goes in this repository: no real logs, locations or names. Tests use synthetic or anonymised logs.

## License

AGPL-3.0.
