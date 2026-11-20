# Groups

Groups divide an organisation's work, such as a "North team" and a "South team". Every flight, session, drone and controller is in one group, and people get their roles in groups. Only admins see this page.

A new organisation has one group, **General**. While it is the only group, FleetRanger doesn't show groups anywhere else: roles are given on [Users](05-users.md), and lists, filters and pickers have no group. Groups show up once you create a second one.

## Roles

| Role       | What it allows                                                                                                                                                            |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Viewer** | Sees the group's flights, sessions and equipment.                                                                                                                         |
| **Pilot**  | Flies for the group: appears in the controller's list of pilots, and pairs controllers. Sees their own flights only.                                                      |
| **Editor** | Sees and organises the group's flights and sessions: sets pilots, forms sessions, writes comments, archives flights, and names drones and controllers. Pairs controllers. |
| **Admin**  | Manages the whole organisation: all groups, users, areas and settings.                                                                                                    |

Roles add up: someone who flies and organises the group's work gets **Pilot** and **Editor**. A pilot who should see the rest of the group's flights also gets **Viewer** or **Editor**.

**Admin** is a role on the whole organisation, not in a group. It doesn't put someone in the list of pilots: an admin who flies also needs **Pilot** in a group.

## Create a group

Select **New group**, enter its **Name** and select **Create**. Then add its members on the group's page.

## A group's page

Select a group to see its members and their roles.

- **Add member**: choose a **User** and at least one role.
- **Edit roles**, the pencil on a member's row, changes their roles in the group.
- **Remove member** takes away their roles in the group. Their flights stay as they are.
- **Edit group** renames it.

Someone can be a member of several groups, with different roles in each. Their roles can also be changed from their page in [Users](05-users.md).

## Move work between groups

Admins move flights, sessions and equipment to another group from their own pages: see [Flights](04-flights.md), [Sessions](03-sessions.md#change-a-session) and [Equipment](08-equipment.md). A flight's pilot and its group are separate: moving a flight doesn't change its pilot.

## Delete a group

Select **Delete group** on the group's page. If the group still holds flights, sessions or equipment, choose where they go under **Move contents to**. Its members lose their roles in it. The last group can't be deleted.

## Who may do what

Only an **Admin** sees **Groups**, creates, renames and deletes groups, and gives roles.
