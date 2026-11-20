# Updates and backups

## Updates

Each release lists what changed in its [release notes](https://github.com/bartslinger/FleetRanger/releases). To update, back up the database (see below), set the new version in `.env` and restart:

```
docker compose pull
docker compose up -d
```

The backend updates the database tables as it starts. Going back to an older release means restoring the backup made before the update.

Images also carry the tag `latest`. We advise against it: an update would then happen at any pull, without a backup made just before.

## Backups

Back up two things: the database and the flight logs in `data/flightlogs` (or your S3 bucket).

```
docker compose exec -T postgres pg_dump -U fleetranger -Fc fleetranger > fleetranger.dump
```

To restore, run the release that made the dump and replace the database with it:

```
docker compose stop backend web
docker compose exec -T postgres dropdb -U fleetranger fleetranger
docker compose exec -T postgres createdb -U fleetranger fleetranger
docker compose exec -T postgres pg_restore -U fleetranger -d fleetranger --no-owner --exit-on-error < fleetranger.dump
docker compose start backend web
```
