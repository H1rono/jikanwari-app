#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Principal {
    Anonymous,
    User(domain::UserId),
}

impl crate::Service {
    pub(crate) fn principal(&self) -> Principal {
        Principal::Anonymous
    }
}

impl crate::AuthenticatedService {
    pub(crate) fn principal(&self) -> Principal {
        Principal::User(self.user_id)
    }
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

// MARK: UserAccessControl

pub trait UserAccessControl<Context, E: domain::Error>: Send + Sync {
    fn judge_get_user(
        &self,
        ctx: Context,
        by: Principal,
        user_id: domain::UserId,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_list_users(
        &self,
        ctx: Context,
        by: Principal,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_create_user(
        &self,
        ctx: Context,
        by: Principal,
        params: &domain::CreateUserParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_update_user(
        &self,
        ctx: Context,
        by: Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;
}

impl<A, C, E> UserAccessControl<C, E> for &A
where
    A: UserAccessControl<C, E>,
    E: domain::Error,
{
    fn judge_get_user(
        &self,
        ctx: C,
        by: Principal,
        user_id: domain::UserId,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_get_user(self, ctx, by, user_id)
    }

    fn judge_list_users(
        &self,
        ctx: C,
        by: Principal,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_list_users(self, ctx, by)
    }

    fn judge_create_user(
        &self,
        ctx: C,
        by: Principal,
        params: &domain::CreateUserParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_create_user(self, ctx, by, params)
    }

    fn judge_update_user(
        &self,
        ctx: C,
        by: Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_update_user(self, ctx, by, user_id, params)
    }
}

pub trait ProvideUserAccessControl: Send + Sync {
    type Context<'a>
    where
        Self: 'a;
    type Error: domain::Error;
    type UserAccessControl<'a>: UserAccessControl<Self::Context<'a>, Self::Error>
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_>;
    fn user_access_control(&self) -> &Self::UserAccessControl<'_>;

    fn judge_get_user(
        &self,
        by: Principal,
        user_id: domain::UserId,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.user_access_control().judge_get_user(ctx, by, user_id)
    }

    fn judge_list_users(
        &self,
        by: Principal,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.user_access_control().judge_list_users(ctx, by)
    }

    fn judge_create_user(
        &self,
        by: Principal,
        params: &domain::CreateUserParams,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.user_access_control()
            .judge_create_user(ctx, by, params)
    }

    fn judge_update_user(
        &self,
        by: Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.user_access_control()
            .judge_update_user(ctx, by, user_id, params)
    }
}

impl<A> ProvideUserAccessControl for &A
where
    A: ProvideUserAccessControl,
{
    type Context<'a>
        = A::Context<'a>
    where
        Self: 'a;
    type Error = A::Error;
    type UserAccessControl<'a>
        = A::UserAccessControl<'a>
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        A::context(self)
    }

    fn user_access_control(&self) -> &Self::UserAccessControl<'_> {
        A::user_access_control(self)
    }
}

// MARK: GroupAccessControl

pub trait GroupAccessControl<Context, E: domain::Error>: Send + Sync {
    fn judge_get_group(
        &self,
        ctx: Context,
        by: Principal,
        group_id: domain::GroupId,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_list_groups(
        &self,
        ctx: Context,
        by: Principal,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_create_group(
        &self,
        ctx: Context,
        by: Principal,
        params: &domain::CreateGroupParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_update_group(
        &self,
        ctx: Context,
        by: Principal,
        group_id: domain::GroupId,
        params: &domain::UpdateGroupParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send;

    fn judge_update_group_members(
        &self,
        ctx: Context,
        by: Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> impl Future<Output = Result<Judgement, E>> + Send;
}

impl<A, C, E> GroupAccessControl<C, E> for &A
where
    A: GroupAccessControl<C, E>,
    E: domain::Error,
{
    fn judge_get_group(
        &self,
        ctx: C,
        by: Principal,
        group_id: domain::GroupId,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_get_group(self, ctx, by, group_id)
    }

    fn judge_list_groups(
        &self,
        ctx: C,
        by: Principal,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_list_groups(self, ctx, by)
    }

    fn judge_create_group(
        &self,
        ctx: C,
        by: Principal,
        params: &domain::CreateGroupParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_create_group(self, ctx, by, params)
    }

    fn judge_update_group(
        &self,
        ctx: C,
        by: Principal,
        group_id: domain::GroupId,
        params: &domain::UpdateGroupParams,
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_update_group(self, ctx, by, group_id, params)
    }

    fn judge_update_group_members(
        &self,
        ctx: C,
        by: Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> impl Future<Output = Result<Judgement, E>> + Send {
        A::judge_update_group_members(self, ctx, by, group_id, members)
    }
}

pub trait ProvideGroupAccessControl: Send + Sync {
    type Context<'a>
    where
        Self: 'a;
    type Error: domain::Error;
    type GroupAccessControl<'a>: GroupAccessControl<Self::Context<'a>, Self::Error>
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_>;
    fn group_access_control(&self) -> &Self::GroupAccessControl<'_>;

    fn judge_get_group(
        &self,
        by: Principal,
        group_id: domain::GroupId,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.group_access_control()
            .judge_get_group(ctx, by, group_id)
    }

    fn judge_list_groups(
        &self,
        by: Principal,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.group_access_control().judge_list_groups(ctx, by)
    }

    fn judge_create_group(
        &self,
        by: Principal,
        params: &domain::CreateGroupParams,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.group_access_control()
            .judge_create_group(ctx, by, params)
    }

    fn judge_update_group(
        &self,
        by: Principal,
        group_id: domain::GroupId,
        params: &domain::UpdateGroupParams,
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.group_access_control()
            .judge_update_group(ctx, by, group_id, params)
    }

    fn judge_update_group_members(
        &self,
        by: Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> impl Future<Output = Result<Judgement, Self::Error>> + Send {
        let ctx = self.context();
        self.group_access_control()
            .judge_update_group_members(ctx, by, group_id, members)
    }
}

impl<A> ProvideGroupAccessControl for &A
where
    A: ProvideGroupAccessControl,
{
    type Context<'a>
        = A::Context<'a>
    where
        Self: 'a;
    type Error = A::Error;
    type GroupAccessControl<'a>
        = A::GroupAccessControl<'a>
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        A::context(self)
    }

    fn group_access_control(&self) -> &Self::GroupAccessControl<'_> {
        A::group_access_control(self)
    }
}
