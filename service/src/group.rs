use domain::{
    CreateGroupParams, Group, GroupCore, GroupId, GroupService, UpdateGroupParams, UserId,
};

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

impl<C, E> GroupService<C, E> for super::Service
where
    C: GroupRepository<E>,
    E: domain::Error,
{
    async fn get_group(&self, ctx: C, id: GroupId) -> Result<Group, E> {
        ctx.get_group(id).await
    }

    async fn list_groups(&self, ctx: C) -> Result<Vec<GroupCore>, E> {
        ctx.list_groups().await
    }

    async fn create_group(&self, ctx: C, params: CreateGroupParams) -> Result<Group, E> {
        ctx.create_group(params).await
    }

    async fn update_group(
        &self,
        ctx: C,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> Result<Group, E> {
        ctx.update_group(id, params).await
    }

    async fn update_group_members(
        &self,
        ctx: C,
        id: GroupId,
        members: &[UserId],
    ) -> Result<Group, E> {
        ctx.update_group_members(id, members).await
    }
}
