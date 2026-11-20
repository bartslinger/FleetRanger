# Flights

**Flights** is the logbook: every flight you can see, one by one. Use it to look up a single flight, and to organise many flights at once.

## Browse flights

**Flights** has the same views as Sessions, chosen at the top right:

- **Cards**: the flights of a week or a month beside a map with their tracks. Use the arrows to go to the previous or next period.
- **Table**: all flights in a list, which you can sort by its columns. This is the view for working on many flights at once.

Above the list you choose the days, and **Filters** opens the other filters: **Pilots**, **Drones**, **Controllers**, and, where they are shown, **Groups** and **Areas flown**. Flights also has:

- **Session**: **All**, or **Without session** for the flights still to be organised.
- **Status**: **Active**, or **Archived** for archived flights.

In the **Pilots** filter, **Unknown pilot** stands for the flights without a pilot.

## A flight

Select a card or a row to open a flight. It shows its pilot, session, takeoff and landing, air time, distance, highest point above takeoff, and the drone, controller, battery and camera it used, with its track on the map. The drone, controller and battery link to their pages in [Equipment](07-equipment.md). Use the arrows to go to the earlier or later flight.

On an open flight, the pencil beside **Pilot** changes its pilot. With several groups, the pencil beside **Group** moves it to another group.

## Organise flights

In the **Table** view, select flights with the checkboxes. **Select all** selects every flight that passes the filters, also those on other pages. Then choose:

- **New session**: the flights become one new session.
- **Add to session**: the flights join a session you choose. The list offers the sessions within 12 hours of them.
- Under the menu beside them: **Remove from session**, **Set pilot**, **Move to group** (with several groups) and **Archive**.

Flights that join a session leave the session they were in. Setting a pilot or moving to a group doesn't change a flight's session.

## Suggested sessions

Most flights join a session by themselves. For the ones that didn't, choose **Session: Without session** in the filters. A **Suggestions** view then appears beside Cards and Table: it groups those flights into suggested sessions, by the session gap.

- Select **New session** on a row to accept a suggestion as it is.
- Or open the suggestion to look first. Each flight has a button to **Leave out** or **Put back**, and **Change pilot** sets the pilot. Select **Create session** to make it, and the page moves on to the next suggestion.

## Archive a flight

Archive a flight that should count as never having happened, such as a test hover. An archived flight is left out of lists, sessions and totals. Only a flight without a session can be archived: remove it from its session first.

Open the flight and select **Archive flight**, or select flights in the table and choose **Archive**. To bring a flight back, filter on **Status: Archived** and select **Restore**. It comes back without a session.

## Who may do what

- Everyone sees their own flights as pilot.
- **Viewer**, **Editor** and **Admin** see the flights of their groups. Admins see every flight.
- **Editor** and **Admin** organise flights into sessions, set pilots, and archive and restore flights.
- Only an **Admin** moves flights to another group.
