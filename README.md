# FleetRanger

Self-hosted flight log analysis for DJI Enterprise drone fleets.

- `backend/` – Rust (axum) API: ingest, decrypt and analyse DJI flight logs
- `web/` – SvelteKit front-end
- `android/` – Kotlin app for DJI remote controllers: pilot selection and log sync
- `deploy/` – container images and deployment

Licensed under the GNU Affero General Public License v3.0, see [LICENSE](LICENSE).

## Development

Requirements: Docker, [rustup](https://rustup.rs) (the toolchain version is pinned in `backend/rust-toolchain.toml`), and [watchexec](https://github.com/watchexec/watchexec) 2.8.0 (`cargo install --locked watchexec-cli --version 2.8.0`).

Start PostgreSQL:

```
docker compose -f compose.dev.yaml up -d
```

Run the backend, rebuilding and restarting on every change:

```
cd backend
DATABASE_URL=postgres://fleetranger:fleetranger@127.0.0.1/fleetranger \
  watchexec -r -w src -w migrations -- cargo run --locked
```

The backend listens on http://127.0.0.1:8080.
