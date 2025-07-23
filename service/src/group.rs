use domain::{
    CreateGroupParams, Group, GroupCore, GroupId, GroupService, UpdateGroupParams, UserId,
};

use crate::rbac::ProvideGroupAccessControl;

// MARK: GroupRepository

pub trait GroupRepository<Context, E: domain::Error>: Send + Sync {
    fn get_group(&self, ctx: Context, id: GroupId)
    -> impl Future<Output = Result<Group, E>> + Send;

    fn list_groups(&self, ctx: Context) -> impl Future<Output = Result<Vec<GroupCore>, E>> + Send;

    fn create_group(
        &self,
        ctx: Context,
        params: CreateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send;

    fn update_group(
        &self,
        ctx: Context,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send;

    fn update_group_members(
        &self,
        ctx: Context,
        id: GroupId,
        members: &[UserId],
    ) -> impl Future<Output = Result<Group, E>> + Send;
}

impl<R, C, E> GroupRepository<C, E> for &R
where
    R: GroupRepository<C, E>,
    E: domain::Error,
{
    fn get_group(&self, ctx: C, id: GroupId) -> impl Future<Output = Result<Group, E>> + Send {
        R::get_group(self, ctx, id)
    }

    fn list_groups(&self, ctx: C) -> impl Future<Output = Result<Vec<GroupCore>, E>> + Send {
        R::list_groups(self, ctx)
    }

    fn create_group(
        &self,
        ctx: C,
        params: CreateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send {
        R::create_group(self, ctx, params)
    }

    fn update_group(
        &self,
        ctx: C,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send {
        R::update_group(self, ctx, id, params)
    }

    fn update_group_members(
        &self,
        ctx: C,
        id: GroupId,
        members: &[UserId],
    ) -> impl Future<Output = Result<Group, E>> + Send {
        R::update_group_members(self, ctx, id, members)
    }
}

pub trait ProvideGroupRepository: Send + Sync {
    type Context<'a>: Send + Sync
    where
        Self: 'a;
    type Error: domain::Error;
    type GroupRepository<'a>: GroupRepository<Self::Context<'a>, Self::Error>
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_>;
    fn group_repository(&self) -> &Self::GroupRepository<'_>;

    fn get_group(&self, id: GroupId) -> impl Future<Output = Result<Group, Self::Error>> + Send {
        let ctx = self.context();
        self.group_repository().get_group(ctx, id)
    }

    fn list_groups(&self) -> impl Future<Output = Result<Vec<GroupCore>, Self::Error>> + Send {
        let ctx = self.context();
        self.group_repository().list_groups(ctx)
    }

    fn create_group(
        &self,
        params: CreateGroupParams,
    ) -> impl Future<Output = Result<Group, Self::Error>> + Send {
        let ctx = self.context();
        self.group_repository().create_group(ctx, params)
    }

    fn update_group(
        &self,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, Self::Error>> + Send {
        let ctx = self.context();
        self.group_repository().update_group(ctx, id, params)
    }

    fn update_group_members(
        &self,
        id: GroupId,
        members: &[UserId],
    ) -> impl Future<Output = Result<Group, Self::Error>> + Send {
        let ctx = self.context();
        self.group_repository()
            .update_group_members(ctx, id, members)
    }
}

// MARK: impl for Service

impl<C, E> GroupService<C, E> for super::Service
where
    C: ProvideGroupRepository<Error = E> + ProvideGroupAccessControl<Error = E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_group(&self, ctx: C, id: GroupId) -> Result<Group, E> {
        ctx.judge_get_group(self.principal(), id)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "Anonymous access denied for group retrieval");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.get_group(id).await.inspect(|g| {
            tracing::debug!(id = %g.id, "Retrieved group");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn list_groups(&self, ctx: C) -> Result<Vec<GroupCore>, E> {
        ctx.judge_list_groups(self.principal())
            .await?
            .allow_or_else(|| {
                tracing::debug!("Anonymous access denied for group listing");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.list_groups().await.inspect(|gs| {
            tracing::debug!(count = gs.len(), "Listed groups");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn create_group(&self, ctx: C, params: CreateGroupParams) -> Result<Group, E> {
        ctx.judge_create_group(self.principal(), &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!("Anonymous access denied for group creation");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.create_group(params).await.inspect(|g| {
            tracing::debug!(id = %g.id, members = g.members.len(), "Created group");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_group(
        &self,
        ctx: C,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> Result<Group, E> {
        ctx.judge_update_group(self.principal(), id, &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "Anonymous access denied for group update");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.update_group(id, params).await.inspect(|g| {
            tracing::debug!(id = %g.id, "Updated group");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_group_members(
        &self,
        ctx: C,
        id: GroupId,
        members: &[UserId],
    ) -> Result<Group, E> {
        ctx.judge_update_group_members(self.principal(), id, members)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "Anonymous access denied for group members update");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.update_group_members(id, members).await.inspect(|g| {
            tracing::debug!(id = %g.id, members = g.members.len(), "Updated group members");
        })
    }
}

// MARK: impl for AuthenticatedService

impl<C, E> GroupService<C, E> for super::AuthenticatedService
where
    C: ProvideGroupRepository<Error = E> + ProvideGroupAccessControl<Error = E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_group(&self, ctx: C, id: GroupId) -> Result<Group, E> {
        ctx.judge_get_group(self.principal(), id)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "User access denied for group retrieval");
                E::forbidden("Access forbidden")
            })?;
        ctx.get_group(id).await.inspect(|g| {
            tracing::debug!(id = %g.id, "Retrieved group");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn list_groups(&self, ctx: C) -> Result<Vec<GroupCore>, E> {
        ctx.judge_list_groups(self.principal())
            .await?
            .allow_or_else(|| {
                tracing::debug!("User access denied for group listing");
                E::forbidden("Access forbidden")
            })?;
        ctx.list_groups().await.inspect(|gs| {
            tracing::debug!(count = gs.len(), "Listed groups");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn create_group(&self, ctx: C, params: CreateGroupParams) -> Result<Group, E> {
        ctx.judge_create_group(self.principal(), &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!("User access denied for group creation");
                E::forbidden("Access forbidden")
            })?;
        ctx.create_group(params).await.inspect(|g| {
            tracing::debug!(id = %g.id, members = g.members.len(), "Created group");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_group(
        &self,
        ctx: C,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> Result<Group, E> {
        ctx.judge_update_group(self.principal(), id, &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "User access denied for group update");
                E::forbidden("Access forbidden")
            })?;
        ctx.update_group(id, params).await.inspect(|g| {
            tracing::debug!(id = %g.id, "Updated group");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_group_members(
        &self,
        ctx: C,
        id: GroupId,
        members: &[UserId],
    ) -> Result<Group, E> {
        ctx.judge_update_group_members(self.principal(), id, members)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "User access denied for group members update");
                E::forbidden("Access forbidden")
            })?;
        ctx.update_group_members(id, members).await.inspect(|g| {
            tracing::debug!(id = %g.id, members = g.members.len(), "Updated group members");
        })
    }
}
