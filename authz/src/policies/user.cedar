@id("forbid-anonymous-user")
forbid (
    principal == User::"anonymous",
    action in [Action::"get-user", Action::"list-users", Action::"update-user"],
    resource is User
);

@id("permit-anonymous-user")
permit (
    principal == User::"anonymous",
    action == Action::"create-user",
    resource is User
);

@id("permit-list-users")
permit (
    principal,
    action == Action::"list-users",
    resource is ListUser
);

@id("permit-get-user")
permit (
    principal,
    action == Action::"get-user",
    resource is User
);

@id("permit-update-myself")
permit (
    principal is User,
    action == Action::"update-user",
    resource is User
) when {
    principal.id == resource.id
};
