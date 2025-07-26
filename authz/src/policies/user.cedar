@id("forbid-anonymous-user")
forbid (
    principal == User::"anonymous",
    action in [Action::"get", Action::"list", Action::"update"],
    resource is User
);

@id("permit-anonymous-user")
permit (
    principal == User::"anonymous",
    action == Action::"create",
    resource is User
);

@id("permit-list-users")
permit (
    principal,
    action == Action::"list",
    resource is ListUser
);

@id("permit-get-user")
permit (
    principal,
    action == Action::"get",
    resource is User
);

@id("permit-update-myself")
permit (
    principal is User,
    action == Action::"update",
    resource is User
) when {
    principal.id == resource.id
};
