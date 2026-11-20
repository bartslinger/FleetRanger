# Sessions

A session is one pilot's flights in a row, such as a morning's work. FleetRanger opens on **Sessions**: it is where you see what was flown, and where you keep a record of it.

## How flights form sessions

When a flight log comes in, its flight joins the pilot's session if it took off less than the session gap after one of that session's flights. Otherwise it starts a new session. The session gap is 2 hours unless an admin changes it under [Administration](09-administration.md).

- A flight without a pilot joins the session of its controller's other flights without a pilot.
- Flights of different pilots never end up in one session by themselves.
- This happens only when a flight first comes in. FleetRanger never undoes a change you made to a session.

## Browse sessions

**Sessions** has two views, chosen at the top right:

- **Cards**: the sessions of a week or a month, beside a map with their tracks. Use the arrows to go to the previous or next period, and the period's menu to show by week or by month. Each card shows the day, a strip with the time of each flight, who flew, and the totals.
- **Table**: all sessions in a list, with pages.

Above the list you choose the days, and **Filters** opens the other filters: **Pilots**, **Drones**, **Controllers**, and, where they are shown, **Groups** and **Areas flown**. Select **Apply** to use them, and **Clear filters** to remove them all. A session is listed when one of its flights passes the filters.

## A session

Select a card, or a row of the table, to open the session. It shows the first takeoff, last landing, duration, air time, distance and number of flights, its pilots, a map of the tracks and a table of its flights. Point at a flight in the table to see its track on the map. Select a flight to open it in [Flights](04-flights.md).

## Change a session

On an open session:

- **Change pilot**, the pencil beside the pilot, sets the pilot of all its flights.
- Select flights in the table for **Change pilot**, **Move to new session** (they become a session of their own) or **Remove from session** (they stay in Flights, without a session).
- **Delete session** deletes the session and its comments. Its flights stay in Flights, without a session.
- With several groups, the pencil beside the group moves the session to another group (**Move to group**). Tick **Also move its flights to this group** to move the flights as well.

To merge sessions, use the **Table** view: select two or more sessions and select **Merge sessions**. Their flights become one session.

To add flights to a session or make a session from flights, use [Flights](04-flights.md#organise-flights).

## Comments

Comments keep a record of what happened in a session, such as the weather or the task. Select **Add comment**, write it and select **Add comment** again. Each comment shows its author and time.

You can edit and delete your own comments from the menu beside them. An edited comment says **Edited**.

> **Note:** The last flight can't be taken out of a session with comments, so the comments aren't lost by accident. To get rid of such a session, delete it: its comments go with it.

## Who may do what

- **Viewer**, **Editor** and **Admin** see the sessions of their groups, with their comments. Admins see every session.
- A **Pilot** without another role sees only their own flights, in the sessions they're in.
- **Editor** and **Admin** change sessions, merge them, delete them and write comments.
- Only an **Admin** moves a session to another group.
