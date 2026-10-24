// What the logged-in user may do, from `GET /api/me`, so pages offer only
// that; the backend still checks every request. The same rule as the
// backend's (policy::Grants): a permission on the organisation counts on
// every group and on what is in no group, one on a group only there. See
// "Groups and roles" in docs/architecture.md.

export type Permission =
	| 'flights.read'
	| 'equipment.read'
	| 'flights.fly_for'
	| 'controllers.pair'
	| 'flights.set_pilot'
	| 'flights.archive'
	| 'sessions.edit'
	| 'equipment.rename'
	| 'flights.move'
	| 'sessions.move'
	| 'equipment.move'
	| 'controllers.manage'
	| 'batteries.rename'
	| 'users.manage'
	| 'groups.manage'
	| 'areas.manage'
	| 'logs.manage';

export interface Permissions {
	/** Held on the whole organisation. */
	organisation: Permission[];
	/** Per group id, what the user's memberships add there. */
	groups: Record<string, Permission[]>;
}

type Holder = { permissions: Permissions } | null | undefined;

/** Whether the user may do this on a group, or with null on what is in no
 * group (the organisation itself, the queue). */
export function may(holder: Holder, permission: Permission, group: string | null): boolean {
	if (!holder) return false;
	const { organisation, groups } = holder.permissions;
	return (
		organisation.includes(permission) ||
		(group !== null && (groups[group]?.includes(permission) ?? false))
	);
}

/** Whether the user may do this on every one of these groups. */
export function mayOnAll(holder: Holder, permission: Permission, groups: (string | null)[]) {
	return groups.every((group) => may(holder, permission, group));
}

/** Whether the user holds the permission anywhere: on a group or the organisation. */
export function anywhere(holder: Holder, permission: Permission): boolean {
	if (!holder) return false;
	const { organisation, groups } = holder.permissions;
	return (
		organisation.includes(permission) ||
		Object.values(groups).some((held) => held.includes(permission))
	);
}

/** Whether the user sees the flights of more than one group: the lists then
 * filter by group. */
export function readsSeveralGroups(holder: Holder): boolean {
	if (!holder) return false;
	const { organisation, groups } = holder.permissions;
	return (
		organisation.includes('flights.read') ||
		Object.values(groups).filter((held) => held.includes('flights.read')).length > 1
	);
}
