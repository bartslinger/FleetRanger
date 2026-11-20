# Configuration

FleetRanger reads its configuration from `.env`, next to `compose.yaml`. After a change, apply it with `docker compose up -d`.

## Required

| Setting                                               | What it is                                                                                            |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `FLEETRANGER_VERSION`                                 | The release to run, such as `v0.20.0`.                                                                |
| `FLEETRANGER_DATA_DIR`                                | Where the database, the flight logs and the controller app are kept.                                  |
| `FLEETRANGER_PUBLIC_URL`                              | The address users open, such as `https://fleet.example.org`.                                          |
| `POSTGRES_PASSWORD`, `POSTGRES_APP_PASSWORD`          | Long random values, such as from `openssl rand -hex 32`.                                              |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | The login provider, see [Requirements](01-requirements.md#login).                                     |
| `DJI_APP_KEY`                                         | Your DJI app key, see [Requirements](01-requirements.md#dji-app-key). Without it no log is processed. |

## Optional

| Setting                              | What it is                                                                                                                                |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `FLEETRANGER_UID`, `FLEETRANGER_GID` | The owner of the flight log directory, which the backend runs as. 65532 by default.                                                       |
| `MAPBOX_ACCESS_TOKEN`                | A public Mapbox token, for the maps.                                                                                                      |
| `FLEETRANGER_SUPPORT_EMAIL`          | Whom pilots contact when the controller app can't find the flight logs.                                                                   |
| `FLEETRANGER_ERROR_REPORTS`          | A Sentry DSN that controllers send their crashes and errors to. None by default.                                                          |
| `FLEETRANGER_MCP`                    | `on` lets AI assistants read summaries of flights and sessions, as the user they act for. Off by default, and then also per organisation. |
| `FLEETRANGER_API`                    | `on` opens the public API at `/api/v1` to apps an admin trusts. Off by default.                                                           |
| `FLEETRANGER_S3_BUCKET`              | Keep the raw logs in an S3 bucket instead of the data directory, with the usual `AWS_*` settings.                                         |
| `RUST_LOG`                           | The backend's log level, `info` by default.                                                                                               |

## Organisation settings

The organisation's name, time zone and session gap are set by its admins in the web app, under Administration. The command-line tool sets them too, along with whether AI assistants may read the organisation:

```
docker compose exec backend fleetranger organisations
docker compose exec backend fleetranger set-organisation <id> --mcp on
```
