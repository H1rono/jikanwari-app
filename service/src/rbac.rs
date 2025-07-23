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

impl Judgement {
    pub fn from_bool(allow: bool) -> Self {
        if allow {
            Judgement::Allow
        } else {
            Judgement::Deny
        }
    }

    pub fn into_bool(self) -> bool {
        match self {
            Judgement::Allow => true,
            Judgement::Deny => false,
        }
    }

    pub(crate) fn allow_or_else<F, E>(self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Judgement::Allow => Ok(()),
            Judgement::Deny => Err(f()),
        }
    }
}

impl From<bool> for Judgement {
    fn from(allow: bool) -> Self {
        Self::from_bool(allow)
    }
}

impl From<Judgement> for bool {
    fn from(judgement: Judgement) -> Self {
        judgement.into_bool()
    }
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
