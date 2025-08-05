// 認証を受けていないユーザーはグループに関して何もできない
@id("forbid-anonymous-user-about-group")
forbid (
    principal == User::"anonymous",
    action,
    resource
) when {
    resource is Group
    || resource is CreateGroup
    || resource is ListGroups
};

@id("permit-get-group")
permit (
    principal,
    action == Action::"get-group",
    resource is Group,
);

@id("permit-list-gruops")
permit (
    principal,
    action == Action::"list-groups",
    resource is ListGroups,
);

// グループ作成時にはメンバーに自身が含まれている必要がある
@id("permit-create-group")
permit (
    principal,
    action == Action::"create-group",
    resource is CreateGroup,
) when {
    // principal: { id }
    // resource: { members: { id }[] }
    resource.members.contains(principal.id)
}

// メンバーに自身が含まれているグループのみ編集(現状, 改名のみ)できる
@id("permit-update-group")
permit (
    principal in ?principal,
    action == Action::"update-gruop",
    resource is Group,
);

// メンバーに自身が含まれているグループのみ他のメンバーを編集できる
// ?principal: 編集前のグループ
// resource: 編集後のグループ { members: { id }[] }
@id("permit-update-group-members")
permit (
    principal in ?principal,
    action = Action::"update-group-members",
    resource is Group
) when {
    resource.members.contains(principal.id)
};
