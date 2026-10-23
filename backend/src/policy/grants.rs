//! Permissions, the built-in roles, and the roles an actor holds; see "Groups
//! and roles" in docs/architecture.md.

use uuid::Uuid;

/// What someone may do, fixed in code. A permission is held on a group, or on
/// the whole organisation, where it covers every group as well.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    FlightsRead,
    EquipmentRead,
    FlightsFlyFor,
    ControllersPair,
    FlightsSetPilot,
    FlightsArchive,
    SessionsEdit,
    EquipmentRename,
    FlightsMove,
    SessionsMove,
    EquipmentMove,
    /// Unpairing controllers, and seeing and revoking others' pairing codes.
    ControllersManage,
    // Batteries, areas, users and logs have no group, so these count only on
    // the whole organisation.
    BatteriesRename,
    UsersManage,
    GroupsManage,
    AreasManage,
    /// Seeing how log processing goes.
    LogsManage,
}

impl Permission {
    pub const ALL: [Self; 17] = [
        Self::FlightsRead,
        Self::EquipmentRead,
        Self::FlightsFlyFor,
        Self::ControllersPair,
        Self::FlightsSetPilot,
        Self::FlightsArchive,
        Self::SessionsEdit,
        Self::EquipmentRename,
        Self::FlightsMove,
        Self::SessionsMove,
        Self::EquipmentMove,
        Self::ControllersManage,
        Self::BatteriesRename,
        Self::UsersManage,
        Self::GroupsManage,
        Self::AreasManage,
        Self::LogsManage,
    ];

    /// Its name in docs/architecture.md and the API, such as `flights.read`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FlightsRead => "flights.read",
            Self::EquipmentRead => "equipment.read",
            Self::FlightsFlyFor => "flights.fly_for",
            Self::ControllersPair => "controllers.pair",
            Self::FlightsSetPilot => "flights.set_pilot",
            Self::FlightsArchive => "flights.archive",
            Self::SessionsEdit => "sessions.edit",
            Self::EquipmentRename => "equipment.rename",
            Self::FlightsMove => "flights.move",
            Self::SessionsMove => "sessions.move",
            Self::EquipmentMove => "equipment.move",
            Self::ControllersManage => "controllers.manage",
            Self::BatteriesRename => "batteries.rename",
            Self::UsersManage => "users.manage",
            Self::GroupsManage => "groups.manage",
            Self::AreasManage => "areas.manage",
            Self::LogsManage => "logs.manage",
        }
    }
}

/// A built-in role: a named set of permissions. Each includes the permissions
/// it depends on, so any single role makes sense on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Viewer,
    Pilot,
    Editor,
    /// Only on the whole organisation.
    Admin,
}

impl Role {
    const ALL: [Self; 4] = [Self::Viewer, Self::Pilot, Self::Editor, Self::Admin];

    /// A role as stored in `group_members` and `organisation_roles`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.as_str() == value)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Pilot => "pilot",
            Self::Editor => "editor",
            Self::Admin => "admin",
        }
    }

    /// The table in docs/architecture.md. A pilot doesn't read the group's
    /// flights: they see their own anyway. Admin has every permission but
    /// flying.
    pub fn grants(self, permission: Permission) -> bool {
        use Permission::*;
        match self {
            Self::Viewer => matches!(permission, FlightsRead | EquipmentRead),
            Self::Pilot => matches!(permission, FlightsFlyFor | ControllersPair),
            Self::Editor => matches!(
                permission,
                FlightsRead
                    | EquipmentRead
                    | ControllersPair
                    | FlightsSetPilot
                    | FlightsArchive
                    | SessionsEdit
                    | EquipmentRename
            ),
            // Flying puts someone in pilot lists: an admin who flies also
            // gets Pilot.
            Self::Admin => permission != FlightsFlyFor,
        }
    }
}

/// The stored names of the roles that grant a permission, for queries that
/// look at memberships, such as pilot lists.
pub fn roles_granting(permission: Permission) -> Vec<String> {
    Role::ALL
        .into_iter()
        .filter(|role| role.grants(permission))
        .map(|role| role.as_str().to_owned())
        .collect()
}

/// The roles an actor holds: on the whole organisation, and per group through
/// their memberships. Their rights are the union of these; nothing denies.
#[derive(Clone, Debug, Default)]
pub struct Grants {
    organisation: Vec<Role>,
    groups: Vec<(Uuid, Role)>,
}

/// The groups where an actor holds a permission, for filtering lists.
pub struct Scope {
    /// Held on the whole organisation, so on every group and on what is in
    /// no group.
    pub all: bool,
    /// Held through a membership, sorted.
    pub groups: Vec<Uuid>,
}

impl Grants {
    /// From the stored rows: a role and its group, or None for the whole
    /// organisation. None if a role is unknown.
    pub fn from_stored(rows: impl IntoIterator<Item = (Option<Uuid>, String)>) -> Option<Self> {
        let mut grants = Self::default();
        for (group, role) in rows {
            let role = Role::parse(&role)?;
            match group {
                Some(group) => grants.groups.push((group, role)),
                None => grants.organisation.push(role),
            }
        }
        Some(grants)
    }

    /// The check: whether the actor may do this on a group, or with None on
    /// what is directly in the organisation. A role on the organisation
    /// counts on every group.
    pub fn may(&self, permission: Permission, group: Option<Uuid>) -> bool {
        self.organisation.iter().any(|role| role.grants(permission))
            || group.is_some_and(|group| {
                self.groups
                    .iter()
                    .any(|(id, role)| *id == group && role.grants(permission))
            })
    }

    /// The check on every group involved, such as all flights of a batch.
    pub fn may_on_all(&self, permission: Permission, groups: &[Option<Uuid>]) -> bool {
        groups.iter().all(|group| self.may(permission, *group))
    }

    /// Whether the actor holds the permission on any group or the
    /// organisation.
    pub fn anywhere(&self, permission: Permission) -> bool {
        let scope = self.scope(permission);
        scope.all || !scope.groups.is_empty()
    }

    /// The list filter: where the actor holds the permission.
    pub fn scope(&self, permission: Permission) -> Scope {
        let mut groups: Vec<Uuid> = self
            .groups
            .iter()
            .filter(|(_, role)| role.grants(permission))
            .map(|(group, _)| *group)
            .collect();
        groups.sort_unstable();
        groups.dedup();
        Scope {
            all: self.organisation.iter().any(|role| role.grants(permission)),
            groups,
        }
    }

    /// The permissions held on the whole organisation, and per group those
    /// its memberships add, so a client can show only what the actor may
    /// do. A group's rights are the organisation's and its own together.
    pub fn permissions(&self) -> (Vec<Permission>, Vec<(Uuid, Vec<Permission>)>) {
        let held = |roles: &[Role]| -> Vec<Permission> {
            Permission::ALL
                .into_iter()
                .filter(|permission| roles.iter().any(|role| role.grants(*permission)))
                .collect()
        };
        let organisation = held(&self.organisation);
        let mut ids: Vec<Uuid> = self.groups.iter().map(|(group, _)| *group).collect();
        ids.sort_unstable();
        ids.dedup();
        let groups = ids
            .into_iter()
            .map(|id| {
                let roles: Vec<Role> = self
                    .groups
                    .iter()
                    .filter(|(group, _)| *group == id)
                    .map(|(_, role)| *role)
                    .collect();
                let own = held(&roles)
                    .into_iter()
                    .filter(|permission| !organisation.contains(permission))
                    .collect();
                (id, own)
            })
            .collect();
        (organisation, groups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NORTH: Uuid = Uuid::from_u128(1);
    const SOUTH: Uuid = Uuid::from_u128(2);

    fn grants(rows: &[(Option<Uuid>, &str)]) -> Grants {
        Grants::from_stored(rows.iter().map(|(group, role)| (*group, role.to_string()))).unwrap()
    }

    /// A role on a group counts there only, never on another group or on
    /// what is directly in the organisation (the queue); a role on the
    /// organisation counts everywhere.
    #[test]
    fn roles_count_where_they_are_held() {
        let editor = grants(&[(Some(NORTH), "editor"), (Some(SOUTH), "pilot")]);
        assert!(editor.may(Permission::FlightsSetPilot, Some(NORTH)));
        assert!(!editor.may(Permission::FlightsSetPilot, Some(SOUTH)));
        assert!(!editor.may(Permission::FlightsSetPilot, None));
        assert!(!editor.may_on_all(Permission::FlightsRead, &[Some(NORTH), Some(SOUTH)]));
        // A pilot doesn't read the group's flights.
        let read = editor.scope(Permission::FlightsRead);
        assert!(!read.all);
        assert_eq!(read.groups, [NORTH]);

        // Only Pilot puts someone in pilot lists, not Admin.
        assert!(!grants(&[(None, "admin")]).anywhere(Permission::FlightsFlyFor));

        let viewer = grants(&[(None, "viewer")]);
        assert!(viewer.may_on_all(Permission::FlightsRead, &[Some(NORTH), None]));
        assert!(!viewer.anywhere(Permission::FlightsArchive));
    }

    /// The web app decides what to offer from `permissions()` with the same
    /// rule as `may`: the organisation's permissions, plus a group's own.
    #[test]
    fn listed_permissions_agree_with_the_check() {
        let mixed = grants(&[
            (None, "viewer"),
            (Some(NORTH), "editor"),
            (Some(NORTH), "pilot"),
        ]);
        let (organisation, groups) = mixed.permissions();
        for permission in Permission::ALL {
            for group in [None, Some(NORTH), Some(SOUTH)] {
                let listed = organisation.contains(&permission)
                    || group.is_some_and(|group| {
                        groups
                            .iter()
                            .any(|(id, held)| *id == group && held.contains(&permission))
                    });
                assert_eq!(
                    listed,
                    mixed.may(permission, group),
                    "{permission:?} on {group:?}"
                );
            }
        }
    }
}
