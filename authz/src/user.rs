#![expect(unused)]

impl<C, E> service::UserAccessControl<C, E> for crate::Engine
where
    C: Send + Sync,
    E: domain::Error,
{
    async fn judge_get_user(
        &self,
        ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_list_users(
        &self,
        ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_create_user(
        &self,
        ctx: C,
        by: service::Principal,
        params: &domain::CreateUserParams,
    ) -> Result<service::Judgement, E> {
        todo!()
    }

    async fn judge_update_user(
        &self,
        ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> Result<service::Judgement, E> {
        todo!()
    }
}
