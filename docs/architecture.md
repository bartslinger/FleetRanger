# Architecture decisions

The decisions made so far, each with a short reason. Open questions are listed in [roadmap.md](roadmap.md).

## Deployment

- **One install per organisation.** Flight data is sensitive, so each organisation runs its own server. Several installs may share one DJI app key.
- **Docker Compose** with three containers: `backend` (Rust), `web` (SvelteKit), `postgres`. TLS and certificates are handled by nginx on the host, which proxies to containers bound to `127.0.0.1`.
- **Images are built on a developer machine** and loaded onto the server. Servers can be small, and compiling Rust on them is impractical. Images target x86_64 and are cross-compiled rather than emulated.
- **Images ship without a registry.** `deploy/ship-image.sh` builds an image tagged with the commit hash (refusing to build from a dirty tree) and loads it on the server over SSH.
- **Internal secrets are generated on the server** (database password, signing keys) and never leave it. External secrets (DJI app key, OIDC client secret) are kept in an Ansible vault.
- **Remote controllers sync over the internet** to the install's public HTTPS endpoint.

## Data

- **PostgreSQL**, without PostGIS for now. Flight tracks are stored as GeoJSON LineStrings in a `jsonb` column on `flights`. Postgres keeps large values out of line, so listings that don't select the track stay light. Moving to PostGIS later is a single backfill with `ST_GeomFromGeoJSON`.
- **Raw logs are files on disk**, never modified after upload. The database records their SHA-256 hash and origin, and identical files are stored once. Existing files are registered where they are; new uploads are stored as `uploads/<sha256>.txt`.
- **Raw logs and flights are separate tables.** A raw log is a file and its processing state; a flight is what was derived from it. A log without a takeoff has no flight, and a log is assumed to hold at most one flight until one turns up with several. Re-processing updates a flight in place, so what is attached to it (pilot, session) survives.
- **Flight tracks are kept at one point per second.** More detail can be derived from the raw log when needed.
- **Columns only for what features use**: shown in lists, sorted, filtered, summed or needed by access rules. Display-only summaries go in a `metrics` jsonb column, typed in code, and are promoted to a column when a feature needs one. `metrics` never holds time series. Everything else stays in the raw log until a feature needs it, then logs are re-processed.
- **Statistics are computed by FleetRanger from the frames**, not taken from the summary DJI's app writes into the log, so they match the track and the method is known. "In the air" is the flight controller's on-ground flag: a log with no frame in the air is not a flight, and air time runs from lifting off to touching down. DJI records logs only while the motors run, and its timer counts motor time, which is typically 2–3 seconds longer per flight.
- **Raw logs and full telemetry are not exposed** to any user, admins included. They exist so logs can be re-processed when the parser improves. Users only see derived data (times, distance, track, statistics).
- **Every DJI keychain request is recorded** per log: the exact request sent (API key redacted), the time, the response status and the keys returned.

## Identity and access

- **Login via OpenID Connect**, with Google first. Sessions are stored on the server, so they can be revoked immediately and audit records reference them.
- **The backend runs the login**: authorization code flow with PKCE, the provider found through its issuer URL. One provider per install for now.
- **No open sign-up.** An admin adds an account by email. The first login with that verified email ties the account to the provider's permanent subject ID, and later logins match on that ID only, so a reassigned or renamed address can't take over or lock out an account. The first account of an install is added with `fleetranger add-account --admin <email>`.
- **Login sessions** (named so to tell them apart from flight sessions) are a random token in the cookie; the database stores only its SHA-256 hash. They expire after 30 days without use. Disabling an account revokes its sessions; accounts are never deleted, so the audit log can refer to them.
- **Pilots and accounts are separate.** A pilot can be created on a remote controller, offline too, with an ID generated on the controller. Admins can also create pilots ahead of time. Admins link a login account to a pilot (at most one pilot per account); invite links come later. Merging duplicate pilots comes with the remote controller app, where duplicates arise.
- **Teams.** Each pilot is in exactly one team. Membership is still a separate table, with a uniqueness constraint, so multiple teams remain possible later.
- **Archived, not deleted.** Pilots and teams that are no longer active are archived, so flights and history keep referring to them.
- **Roles** are one per account: member, all flights, or admin.
- **Flights** have a team and an optional pilot. The team is the pilot's team at flight time. A flight with no pilot takes the remote controller's team, or else goes to an organisation-level queue. Changing a pilot's team does not move their past flights.
- **Drones and remote controllers** belong to the organisation. Each can have a team and a current holder (a pilot). The holder is informational only and grants no access. They are registered automatically from the serials in imported logs; flights keep the serials as text.
- **One table per kind of equipment** (`aircraft`, `remote_controllers`, later batteries), not one devices table: they share a few columns but will differ, for example controllers get enrollment and sync settings, and only a controller's team decides the team of a flight without a pilot.
- **Who sees a flight:**
  - its pilot
  - members of its team
  - participants of an operation containing it
  - the all-flights role
  - admins
  - later, people it has been shared with
- **Within a team**, any member can change a flight's pilot to another member, or to nobody. No acceptance is needed. "This wasn't me" means setting the pilot to nobody, and the flight stays in the team.
- **Admins** manage users, pilots, teams and devices, and can move flights between teams.
- **Setting a flight's pilot:**
  - Members can change flights of their pilot's current team, choosing a pilot of that team or nobody.
  - Admins can choose any active pilot, and the flight's team follows the pilot, also when changing from one pilot to another. That keeps "the team is the pilot's team" true and empties the organisation's queue in one step.
  - When an admin moves a flight to another team without choosing a pilot, the pilot is cleared unless they are in the new team. A flight never has a pilot from another team.
  - The all-flights role only views.
  - These changes work on many flights at once (batch actions in the logbook). A batch is applied whole or not at all.
- **All access decisions go through one policy module** in the backend, never through ad-hoc checks in handlers. Service functions take an `Actor` (who is acting), so a handler can't skip the check.
- **Simplicity first.** The visibility condition is written directly in the flight queries, with a test that list and single-flight access agree. It gets extracted (for example into a SQL function) only when duplication becomes a problem.
- **Audit log:** append-only, recording every view, change, reassignment, move and merge, with the user, session, time and IP. There is no hash chain.

## Sessions and operations

- **A session is one pilot's consecutive flights**, for example a morning shift. A remote controller often powers off during a session while the pilot relocates, so power cycles are not boundaries. Sessions are grouped automatically by a time-gap rule: a flight joins its pilot's session when it starts within the gap (2 hours, configurable per install), and otherwise starts a new one. Grouping waits for pilots: a flight without a pilot is in no session, because the remote controller is not a reliable stand-in. Users can move flights between sessions, and split or merge sessions. Automatic grouping only places flights that have no session yet, so it never undoes a user's changes.
- **Selecting a pilot on the remote controller** sets who is flying from that moment on.
- **An operation groups flights across teams**, for example a joint operation. Anyone can create one. Anyone with a flight in an operation sees all of its flights. Flights can only be added by people who can already see them, and must overlap the operation's time window. One operation per flight for now, through a link table, so this can be relaxed later.

## Backend structure

- **One crate, organised by feature** (`flights/`, `sessions/`, `ingest/`, and so on). Each feature has `routes.rs` (HTTP), `service.rs` (policy checks, logic, audit) and `db.rs` (sqlx queries). Cross-cutting concerns (`auth/`, `policy/`, `audit/`) are top-level modules.
- **sqlx** with plain SQL migrations and compile-time checked queries.
- **dji-log-parser is vendored** in `backend/vendor/` with our patches, each explained in `backend/vendor/README.md`. Patches only fix bugs; code we don't use stays as it is. FleetRanger uses the parser to build the DJI key request, but sends it itself so it can store exactly what was sent.
- **Background work** (DJI keychain requests, parsing, re-processing) runs as a loop in the same binary. There is no separate queue: each raw log has a processing status (pending, done, failed) with attempts and the last error. The loop picks up pending logs, and re-processing means setting logs back to pending. CPU-heavy parsing has a concurrency limit.

## Web and API

- nginx routes `/api/*` and `/auth/*` to the backend and everything else to SvelteKit.
- The backend owns login and the session cookie (`HttpOnly`, `Secure`, `SameSite=Lax`). SvelteKit holds no secrets and has no database access. During server-side rendering and in form actions it calls the backend over the internal network, forwarding the browser's cookie and client IP.
- The backend trusts forwarded client IP headers only from the internal network.
- **Times are shown in the organisation's time zone**, one setting per install (`TIME_ZONE` on the web app for now). Everyone sees the same times, as in reports, and server and browser render the same text.

## Clients

- **Backend:** Rust with axum.
- **Web:** SvelteKit with server-side rendering (Node), TypeScript, pnpm, Prettier and ESLint. The UI follows the Carbon Design System through our own components; see [design-system.md](design-system.md).
- **Remote controller app:** Kotlin, for DJI Enterprise controllers running DJI Pilot 2. The controller is configured to launch FleetRanger on boot. After a pilot is selected (default: the last pilot), the app starts DJI Pilot 2. Finished logs sync only over Wi-Fi networks chosen in the app.

## Customer-specific needs

FleetRanger stays generic. Needs specific to one organisation are left out until they are actually built, and are then implemented in the simplest form. They are generalised only when a second organisation needs them. No customer data goes in this repository: no real logs, locations or names. Tests use synthetic or anonymised logs.

## License

AGPL-3.0.
