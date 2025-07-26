@id("forbid-anonymous-user")
forbid (
    principal == User::"anonymous",
    action in [UserAction::"get", UserAction::"list", UserAction::"update"],
    resource is User
);

@id("permit-anonymous-user")
permit (
    principal == User::"anonymous",
    action == UserAcion::"create",
    resource is User
);

@id("permit-list-users")
permit (
    principal,
    action == UserAction::"list",
    resource is ListUser
);

@id("permit-get-user")
permit (
    principal,
    action == UserAction::"get",
    resource is User
);

@id("permit-update-myself")
permit (
    principal is User,
    action == UserAction::"update",
    resource is User
) when {
    principal.id == resource.id
};
