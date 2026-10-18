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
- **Everyone is a user.** Pilots, viewers and admins are all users, with one name entered by an admin. A user flies for a team, can log in, or both. Only admins create users, also for pilots; the remote controller only downloads the list (see "Clients"). That avoids duplicates, so there is no merging of users.
- **Logins are separate from users.** A user can have a login: an email address, tied to the provider's subject ID on first use. Login data (`logins`, `login_sessions`) is kept apart from what a user is, so the security-sensitive part stays small. Removing a login stops someone from logging in while they keep flying.
- **No open sign-up.** An admin gives a user a login by email. The first login with that verified email ties it to the provider's permanent subject ID, and later logins match on that ID only, so a reassigned or renamed address can't take over or lock out a login. The first user of an install is added with `fleetranger add-user --admin <email> <name>`.
- **Login sessions** (named so to tell them apart from flight sessions) are a random token in the cookie; the database stores only its SHA-256 hash. They expire after 30 days without use. Archiving a user or removing their login revokes their sessions.
- **Teams.** A user flies for at most one team; admins and viewers who don't fly have none. Membership is still a separate table, with a uniqueness constraint, so multiple teams remain possible later.
- **Archived, not deleted.** Users and teams that are no longer active are archived, so flights and history keep referring to them. An archived user can't log in and can't be chosen as a pilot. A user without flights can be deleted, for example one created by mistake.
- **Roles** are one per user: member, all flights, or admin.
- **Flights** have a team and an optional pilot. The team is the pilot's team at flight time. A flight with no pilot takes the remote controller's team, or else goes to an organisation-level queue. Changing a pilot's team does not move their past flights.
- **Drones and remote controllers** belong to the organisation. Each can have a team and a current holder (a user). The holder is informational only and grants no access. They are registered automatically from the serials in imported logs; flights keep the serials as text.
- **One table per kind of equipment** (`aircraft`, `remote_controllers`, later batteries), not one devices table: they share a few columns but will differ, for example controllers get enrollment and sync settings, and only a controller's team decides the team of a flight without a pilot.
- **Who sees a flight:**
  - its pilot
  - members of its team
  - participants of an operation containing it
  - the all-flights role
  - admins
  - later, people it has been shared with
- **Within a team**, any member can change a flight's pilot to another member, or to unknown. No acceptance is needed. "This wasn't me" means setting the pilot to unknown, and the flight stays in the team.
- **Admins** manage users, teams and devices, and can move flights between teams.
- **Setting a flight's pilot:**
  - Members can change flights of their own current team, choosing a user of that team or unknown.
  - Admins can choose any active user in a team, and the flight's team follows the pilot, also when changing from one pilot to another. That keeps "the team is the pilot's team" true and empties the organisation's queue in one step.
  - When an admin moves a flight to another team without choosing a pilot, the pilot is cleared unless they are in the new team. A flight never has a pilot from another team.
  - The all-flights role only views.
  - These changes work on many flights at once (batch actions in the logbook). A batch is applied whole or not at all.
- **All access decisions go through one policy module** in the backend, never through ad-hoc checks in handlers. Service functions take an `Actor` (who is acting), so a handler can't skip the check.
- **Simplicity first.** The visibility condition is written directly in the flight queries, with a test that list and single-flight access agree. It gets extracted (for example into a SQL function) only when duplication becomes a problem.
- **Audit log:** append-only, recording every view, change, reassignment, move and merge, with the user, session, time and IP. There is no hash chain.

## Sessions and operations

- **Sessions exist for reporting:** they summarise several flights as one thing.
- **A session is one pilot's consecutive flights**, for example a morning shift. A remote controller often powers off during a session while the pilot relocates, and the pilot may drive several kilometres between flights, so neither power cycles nor distance are boundaries. Sessions are grouped automatically by a time-gap rule: a flight joins its pilot's session when it is less than the gap away from one of its flights (2 hours, `FLEETRANGER_SESSION_GAP_MINUTES`), and otherwise starts a new one. Grouping waits for pilots: a flight without a pilot is in no session, because the remote controller is not a reliable stand-in. Users can move flights between sessions, and split or merge sessions. Automatic grouping only places flights that have no session yet, so it never undoes a user's changes.
- **Different pilots are never grouped automatically.** Several pilots in one session is rare, so it only happens when a user merges sessions. A session therefore stores no pilot: its pilots, teams, times and totals come from its flights.
- **Who changes sessions** (moving flights, splitting, merging) follows setting a flight's pilot: admins change any session, members only sessions whose flights are all in their own team, and the all-flights role none.
- **Merging** puts all flights in the earliest of the sessions and deletes the others. **Splitting** moves selected flights of a session into a new session. Moving flights to another session is splitting them off and merging.
- **Changing a flight's pilot takes it out of its session**, and it is grouped again with the new pilot's flights. Sessions left without flights are deleted.
- **Report fields such as an organisation's "time on" and "time off"** are organisation-specific and are not derived from the flights; they come with session reports.
- **Selecting a pilot on the remote controller** sets who is flying from that moment on.
- **An operation groups flights across teams**, for example a joint operation. Anyone can create one. Anyone with a flight in an operation sees all of its flights. Flights can only be added by people who can already see them, and must overlap the operation's time window. One operation per flight for now, through a link table, so this can be relaxed later.

## Backend structure

- **One crate, organised by feature** (`flights/`, `sessions/`, `ingest/`, and so on). Each feature has `routes.rs` (HTTP), `service.rs` (policy checks, logic, audit) and `db.rs` (sqlx queries). Cross-cutting concerns (`auth/`, `policy/`, `audit/`) are top-level modules.
- **sqlx** with plain SQL migrations and compile-time checked queries.
- **dji-log-parser is vendored** in `backend/vendor/` with our patches, each explained in `backend/vendor/README.md`. Patches only fix bugs; code we don't use stays as it is. FleetRanger uses the parser to build the DJI key request, but sends it itself so it can store exactly what was sent.
- **Background work** (DJI keychain requests, parsing, re-processing) runs as a loop in the same binary. There is no separate queue: each raw log has a processing status (pending, done, failed) with attempts and the last error. The loop picks up pending logs, and re-processing means setting logs back to pending. CPU-heavy parsing has a concurrency limit.
- **Processing progress is shown to admins** in the web app's header. What the loop is doing right now (the log, its step, a pause and why) is kept in memory, since the loop and the API share a process; counts come from the database. A batch is what the loop has processed since it last found nothing pending. The browser polls for it (every few seconds while logs are pending, rarely otherwise) rather than having it pushed.

## Web and API

- nginx routes `/api/*` and `/auth/*` to the backend and everything else to SvelteKit.
- The backend owns login and the session cookie (`HttpOnly`, `Secure`, `SameSite=Lax`). SvelteKit holds no secrets and has no database access. During server-side rendering and in form actions it calls the backend over the internal network, forwarding the browser's cookie and client IP.
- The backend trusts forwarded client IP headers only from the internal network.
- **Maps use Mapbox GL JS**, with Mapbox's satellite-with-streets style by default. Each install has its own public access token (`MAPBOX_ACCESS_TOKEN` on the web app), read at runtime so one image serves every install. The browser loads tiles, styles and fonts from Mapbox, which learns roughly which area is viewed; tracks themselves are drawn in the browser and never sent to Mapbox.
- **Views:** the same data can be shown in different views, depending on what the user wants to do with it. Sessions, for example, are a table, or cards beside one map that shows the selected session. The view is part of the URL.
- **Times are shown in the organisation's time zone**, one setting per install (`TIME_ZONE` on the web app for now). Everyone sees the same times, as in reports, and server and browser render the same text.

## Clients

- **Backend:** Rust with axum.
- **Web:** SvelteKit with server-side rendering (Node), TypeScript, pnpm, Prettier and ESLint. The UI follows the Carbon Design System through our own components; see [design-system.md](design-system.md).
- **Remote controller app:** Kotlin, for DJI Enterprise controllers running DJI Pilot 2. The controller is configured to launch FleetRanger on boot. After a pilot is selected, the app starts DJI Pilot 2. The last pilot is the default and takes one tap; any other pilot takes two ("Other pilot", then the list). The app downloads the users it can offer and keeps the list for offline use; it doesn't create users. A pilot who isn't in the list chooses "Not in the list": the flight has no pilot, goes to the controller's team or the queue, and gets its pilot later in the logbook. Finished logs sync only over Wi-Fi networks chosen in the app (see pairing and Wi-Fi below).
- **Pairing links a controller to the install, and any user can do it.** The app is downloaded on the controller from the install's public download page, which also hands the app the server's address. The app then shows a short code and a QR code that links to the approval page with that code. A logged-in user scans it with their phone (or enters the code), sees which controller it is (model and serial), and approves it. Nobody logs in on the controller, which is shared. The controller then gets a long random token, stored on the server as a hash. The token only allows downloading the pilot list and uploading logs.
- **A code stays valid while the controller shows it.** The controller checks every few seconds whether it has been approved, and each check keeps its request alive, so the code doesn't change while someone types it. About a minute after the controller stops checking (switched off, or the screen closed) the request expires. After an hour without approval the controller stops and offers a new code.
- **Waiting controllers are announced in the web app**, to every user, by a notification in the header that shows the controller (model and serial) but not its code. Approving still needs the code, which proves the approver is at the controller.
- **A paired controller joins the team of the user who approved it**, or stays without a team when that user has none, until an admin sets one. It offers every active pilot, those of its own team first and the rest of the organisation on request.
- **Unpairing** is done on the controller itself or by an admin. The token stops working immediately.
- **Logs upload only over Wi-Fi networks approved on the controller.** The app sees the network's name (on Android this needs the location permission, which DJI Pilot 2 needs anyway) and asks once per new network; approved networks can be removed again. It never uploads over a network Android reports as metered, such as most phone hotspots.
- **The controller app's UI is Jetpack Compose with our own Carbon theme:** Carbon's tokens (the same colours, IBM Plex and spacing as the web app) in a small set of our own components, as on the web. Carbon has no official Android library, and a community one would be a third-party dependency for a handful of screens.
- **The APK is built with Gradle in a pinned x86_64 Docker image.** Google's Android build tools don't run on arm64 Linux, so on an arm64 machine the image runs under QEMU emulation; CI can run it natively.

## Customer-specific needs

FleetRanger stays generic. Needs specific to one organisation are left out until they are actually built, and are then implemented in the simplest form. They are generalised only when a second organisation needs them. No customer data goes in this repository: no real logs, locations or names. Tests use synthetic or anonymised logs.

## License

AGPL-3.0.
