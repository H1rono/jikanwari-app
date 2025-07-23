#![expect(unused)]

impl<C, E> service::GroupAccessControl<C, E> for crate::Engine
where
    C: Send + Sync,
    E: domain::Error,
{
    async fn judge_get_group(
        &self,
        ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_list_groups(
        &self,
        ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_create_group(
        &self,
        ctx: C,
        by: service::Principal,
        params: &domain::CreateGroupParams,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_update_group(
        &self,
        ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
        params: &domain::UpdateGroupParams,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_update_group_members(
        &self,
        ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> Result<service::Judgement, E> {
        todo!()
    }
}
