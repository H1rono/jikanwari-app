#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Principal {
    Anonymous,
    User(domain::UserId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Judgement {
    Allow,
    Deny,
}

pub trait UserAccessControl<E: domain::Error>: Send + Sync {
    fn allow_get_user(
        &self,
        by: Principal,
        user_id: domain::UserId,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_list_users(&self, by: Principal) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_create_user(
        &self,
        by: Principal,
        params: &domain::CreateUserParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_update_user(
        &self,
        by: Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;
}

pub trait GroupAccessControl<E: domain::Error>: Send + Sync {
    fn allow_get_group(
        &self,
        by: Principal,
        group_id: domain::GroupId,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_list_groups(&self, by: Principal)
    -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_create_group(
        &self,
        by: Principal,
        params: &domain::CreateGroupParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_update_group(
        &self,
        by: Principal,
        group_id: domain::GroupId,
        params: &domain::UpdateGroupParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn allow_update_group_members(
        &self,
        by: Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> impl Future<Output = Result<Judgement, E>> + Send;
}
