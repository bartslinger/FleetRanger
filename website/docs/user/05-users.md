# Users

**Users** lists everyone in your organisation: the people who log in, and the pilots who only fly. Only admins see this page.

Everyone is a user, with a name and roles. A user can have a login, can fly, or both: a pilot without a login still appears in the controller's list of pilots.

## Add a user

Select **New user** and fill in:

- **Name**: as it appears in lists and on the controller.
- **Email (optional)**: the address of the account they log in with. Leave it empty for someone who only flies.
- **FleetRanger roles**: what they may do. See [Roles](06-groups.md#roles).

Select **Create**. FleetRanger sends no email: tell the person themselves that they can log in. When they log in for the first time, they see the invitation and select **Accept**. Until then, their page says **Invited, not accepted yet**.

When your organisation has several groups, **New user** offers only **Admin**. Give the other roles in the user's groups, as below.

## A user's page

Select a user to see their flights as pilot, their email, their last login and their roles. With several groups, the page also lists their groups with their roles in each.

- **Edit user** changes their name, email and roles.
- **Add to group**, **Edit roles** (the pencil on a group's row) and **Remove from group** change their groups, with several groups.

Removing someone's email stops them from logging in. They keep their flights and can still fly.

## Archive or delete a user

The menu beside **Edit user** has:

- **Archive user**: for someone who has left. They can't log in, can't be chosen as a pilot and are hidden from lists, but their flights stay as they are. Turn on **Show archived** to see them, and **Restore user** on their page to bring them back.
- **Delete user**: only for a user without flights, such as one added by mistake. This can't be undone.

You can't archive or delete yourself.

## Impersonate a user

**Impersonate user**, in the same menu, shows FleetRanger as that person sees it, for example to help them. A banner at the top of every page says whom you are impersonating. Changes you make are made as them. Select **Stop impersonating** in the banner to go back.

## Who may do what

Only an **Admin** sees **Users** and adds, changes, archives, deletes and impersonates users. The organisation always keeps one admin: the last admin can't lose the **Admin** role or be archived.
