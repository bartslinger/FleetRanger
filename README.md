# FleetRanger

Self-hosted flight log analysis for DJI Enterprise drone fleets.

- `backend/` – Rust (axum) API: ingest, decrypt and analyse DJI flight logs
- `web/` – SvelteKit front-end
- `android/` – Kotlin app for DJI remote controllers: pilot selection and log sync
- `deploy/` – container images and deployment

Licensed under the GNU Affero General Public License v3.0, see [LICENSE](LICENSE).

## Development

Requirements: Docker, [rustup](https://rustup.rs) (the toolchain version is pinned in `backend/rust-toolchain.toml`), [watchexec](https://github.com/watchexec/watchexec) 2.8.0 (`cargo install --locked watchexec-cli --version 2.8.0`), and sqlx-cli 0.9.0 (`cargo install --locked sqlx-cli --version 0.9.0 --no-default-features --features postgres,rustls`).

Start PostgreSQL and the web dev server (SvelteKit with hot reload, in a container):

```
docker compose -f compose.dev.yaml up -d
```

The web app is served on http://127.0.0.1:5173. It proxies `/api` and `/auth` to the backend. If edits don't trigger a reload (file events may not cross a VM's shared folder), start it with `VITE_POLL=1`.

Run web checks inside the container:

```
docker compose -f compose.dev.yaml exec web pnpm check
docker compose -f compose.dev.yaml exec web pnpm lint
```

Run the backend, rebuilding and restarting on every change:

```
cd backend
DATABASE_URL=postgres://fleetranger:fleetranger@127.0.0.1/fleetranger \
FLEETRANGER_BIND=127.0.0.1:8080 \
  watchexec -r -w src -w migrations -- cargo run --locked
```

The backend listens on http://127.0.0.1:8080.

Logging in needs a Google OAuth client (type "Web application") with the redirect URI `http://localhost:5173/auth/callback`. Put `OIDC_CLIENT_ID` and `OIDC_CLIENT_SECRET` in `backend/.env` (see below). Another OpenID Connect provider can be set with `OIDC_ISSUER`, and `FLEETRANGER_PUBLIC_URL` is where browsers reach the app (`http://localhost:5173` by default). Nobody can log in without an account, so add yours first:

```
cargo run --locked -- add-account you@example.org
```

Raw logs are read from `FLEETRANGER_LOG_DIR` (`/data/flightlogs` by default). The backend processes pending logs in the background. Encrypted logs need a DJI app key in `DJI_APP_KEY`; keep it and the OIDC settings in `backend/.env` (ignored by git) and load it with `set -a; . ./.env; set +a` before starting the backend. To register logs that are already in that directory, run the backend with the `import` command:

```
FLEETRANGER_LOG_DIR=~/fleetranger-testlogs cargo run --locked -- import
```

After adding or changing a SQL query, run `cargo sqlx prepare` (with `DATABASE_URL` set) and commit the `.sqlx` directory. Image builds check queries against it, as they have no database.
