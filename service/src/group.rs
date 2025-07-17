use domain::{
    CreateGroupParams, Group, GroupCore, GroupId, GroupService, UpdateGroupParams, UserId,
};

// MARK: GroupRepository

pub trait GroupRepository<E: domain::Error>: Send + Sync {
    fn get_group(&self, id: GroupId) -> impl Future<Output = Result<Group, E>> + Send;

    fn list_groups(&self) -> impl Future<Output = Result<Vec<GroupCore>, E>> + Send;

    fn create_group(
        &self,
        params: CreateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send;

    fn update_group(
        &self,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send;

    fn update_group_members(
        &self,
        id: GroupId,
        members: &[UserId],
    ) -> impl Future<Output = Result<Group, E>> + Send;
}

impl<R, E> GroupRepository<E> for &R
where
    R: GroupRepository<E>,
    E: domain::Error,
{
    fn get_group(&self, id: GroupId) -> impl Future<Output = Result<Group, E>> + Send {
        R::get_group(self, id)
    }

    fn list_groups(&self) -> impl Future<Output = Result<Vec<GroupCore>, E>> + Send {
        R::list_groups(self)
    }

    fn create_group(
        &self,
        params: CreateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send {
        R::create_group(self, params)
    }

    fn update_group(
        &self,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> impl Future<Output = Result<Group, E>> + Send {
        R::update_group(self, id, params)
    }

    fn update_group_members(
        &self,
        id: GroupId,
        members: &[UserId],
    ) -> impl Future<Output = Result<Group, E>> + Send {
        R::update_group_members(self, id, members)
    }
}

// MARK: impl for Service

impl<C, E> GroupService<C, E> for super::Service
where
    C: GroupRepository<E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_group(&self, _ctx: C, id: GroupId) -> Result<Group, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }

    #[tracing::instrument(skip_all)]
    async fn list_groups(&self, _ctx: C) -> Result<Vec<GroupCore>, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }

    #[tracing::instrument(skip_all)]
    async fn create_group(&self, ctx: C, params: CreateGroupParams) -> Result<Group, E> {
        ctx.create_group(params).await.inspect(|g| {
            tracing::debug!(id = %g.id, members = g.members.len(), "Created group");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_group(
        &self,
        _ctx: C,
        id: GroupId,
        _params: UpdateGroupParams,
    ) -> Result<Group, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_group_members(
        &self,
        _ctx: C,
        id: GroupId,
        _members: &[UserId],
    ) -> Result<Group, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }
}

// MARK: impl for AuthenticatedService

impl<C, E> GroupService<C, E> for super::AuthenticatedService
where
    C: GroupRepository<E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_group(&self, ctx: C, id: GroupId) -> Result<Group, E> {
        ctx.get_group(id).await.inspect(|g| {
            tracing::debug!(id = %g.id, "Retrieved group");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn list_groups(&self, ctx: C) -> Result<Vec<GroupCore>, E> {
        ctx.list_groups().await.inspect(|gs| {
            tracing::debug!(count = gs.len(), "Listed groups");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn create_group(&self, ctx: C, params: CreateGroupParams) -> Result<Group, E> {
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
        ctx.update_group_members(id, members).await.inspect(|g| {
            tracing::debug!(id = %g.id, members = g.members.len(), "Updated group members");
        })
    }
}
