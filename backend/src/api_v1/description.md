Read an organisation's sessions, flights and equipment, acting for a user through OAuth, with that user's access: an app sees what its user sees in FleetRanger. Version 1 is read-only and holds no locations: no tracks, area boundaries or takeoff points.

## Access

- **Tokens** come from the install's OAuth, issued for the resource `/api/v1`. Apps the install added skip the consent screen; others ask the user.
- **Scopes** say which resources an app reaches and how deep: `<resource>:list`, `:read`, `:write`, each level including the one below. The user's own access still decides what is in each answer.
- **401** for a missing, expired or wrong token. **403** only when the token lacks a scope the request needs, naming it. **404** for anything the user can't see, as if it didn't exist.

## Conventions

- **JSON, field names in `snake_case`.** Apps ignore fields they don't know, so new fields can be added within v1.
- **References are ids**, such as `"pilot": "0192…"`; `expand` turns them into objects.
- **Times** are instants in UTC, RFC 3339. Render them in the organisation's time zone from `GET /organisation`.
- **Days in filters** (`from`, `to`) are dates in the organisation's time zone, `to` included: "October" is the organisation's October.
- **Units:** durations in whole seconds (`_s`), distances and heights in whole metres (`_m`).
- **Ids** are UUIDs. Serials are attributes, never identifiers.
- **Errors** are a status with `{"message": "…"}`, a sentence for a person.

## Filters

Repeat a filter for several values: values of one filter combine with *or*, different filters with *and*. `?pilot=none&pilot=…&group=…` means flights without a pilot or by that pilot, in that group. `none` stands for "no group", "no pilot" or "no session" where a filter allows it.

Filters decide which sessions are listed, by their flights: a session is listed when one of its flights passes them, and still shows the figures of all the flights the user sees in it.

## Expand

Ask for related objects in the same request with `expand`, comma-separated, as Stripe does. An expanded field holds the object instead of its id, shaped like an item of its own list: with `expand=drone`, a flight's `drone` is the drone. A session's `comments` and `flights` aren't in its base shape; they appear only when expanded.

Expansions nest with a dot, at most three levels deep, and a path expands what leads to it too:

`GET /api/v1/sessions?from=2026-10-01&to=2026-10-31&expand=comments.author,flights.batteries,flights.areas.area`

What the web app shows on a session or flight (its group, pilots, areas and equipment) comes with it, so expanding it needs no further scope. A scope is needed only where an expansion reaches another resource or level: a session's `comments` (`sessions:read`) and `flights` (`flights:list`), a flight's `session` (`sessions:list`). Asking for one the token lacks is refused with a 403, never silently left out.

## Paging

Every list is paged with a cursor. Ask for up to `limit` items (1 to 100, 50 by default); the answer has `items` and `next_cursor`. Pass `next_cursor` back as `cursor`, with the same filters, for the following page; it is null on the last one. The cursor is opaque: don't build or change it. Each list has one fixed order whose keys don't change, so a pass through all pages misses nothing, even while flights are added.
