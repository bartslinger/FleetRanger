# FleetRanger

Self-hosted flight log analysis for DJI Enterprise drone fleets.

- `backend/` – Rust (axum) API: ingest, decrypt and analyse DJI flight logs
- `web/` – SvelteKit front-end
- `android/` – Kotlin app for DJI remote controllers: pilot selection and log sync
- `deploy/` – container images and deployment

Licensed under the GNU Affero General Public License v3.0, see [LICENSE](LICENSE).
