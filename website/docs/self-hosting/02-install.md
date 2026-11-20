# Install

These steps install a release, here v0.20.1; take the latest from the [releases](https://github.com/bartslinger/FleetRanger/releases).

## The containers

In `/srv/fleetranger`, fetch the release's compose file and configuration template:

```
curl -fLO https://raw.githubusercontent.com/bartslinger/FleetRanger/v0.20.1/deploy/compose.yaml
curl -fL -o .env https://raw.githubusercontent.com/bartslinger/FleetRanger/v0.20.1/deploy/.env.example
```

Fill in `.env`, as [Configuration](03-configuration.md) describes. Then create the data directory (`FLEETRANGER_DATA_DIR`, `/srv/fleetranger/data` by default), owned by the user the backend runs as (`FLEETRANGER_UID` and `FLEETRANGER_GID`), and start FleetRanger:

```
mkdir -p data/flightlogs data/android
chown 65532:65532 data/flightlogs
docker compose up -d
```

The backend creates the database tables on its first start, and updates them on every new release.

## HTTPS

FleetRanger's containers listen on localhost only: the backend on port 8080 and the web app on port 3000. A reverse proxy on the host terminates HTTPS and passes requests on. [deploy/nginx.conf](https://github.com/bartslinger/FleetRanger/blob/main/deploy/nginx.conf) is the nginx configuration we run ourselves; set your domain and certificate paths in it. Leave out its `http2 off;` line on nginx older than 1.25.1.

Another proxy works as well, as long as it does what that file does:

- `/api/`, `/auth/`, `/mcp` and `/.well-known/oauth-*` go to the backend, everything else to the web app.
- Uploads of up to 64 MB are accepted under `/api/`.
- `/api/events` is passed on unbuffered and kept open for up to an hour.
- `X-Forwarded-For` is always set, with the client's address last.
- Pairing (`/api/controller/v1/pairing`) and app registration (`/auth/oauth/register`) are limited to 10 requests a minute per address, as pairing codes are short.

## The first admin

Create your organisation and yourself as its admin, with the email address you log in with:

```
docker compose exec backend fleetranger add-organisation "Example Drones" --time-zone Europe/Amsterdam
docker compose exec backend fleetranger add-user --admin pilot@example.org "Alex Example"
```

Then open your address, log in, and accept the invitation. Everything else, such as other users, groups and equipment, is set up in the web app; see [Getting started](../user/01-getting-started.md).
