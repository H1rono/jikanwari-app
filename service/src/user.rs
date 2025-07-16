use domain::{CreateUserParams, UpdateUserParams, User, UserId, UserService};

pub trait UserRepository<E: domain::Error>: Send + Sync {
    fn get_user(&self, id: UserId) -> impl Future<Output = Result<User, E>> + Send;

    fn list_users(&self) -> impl Future<Output = Result<Vec<User>, E>> + Send;

    fn create_user(&self, params: CreateUserParams)
    -> impl Future<Output = Result<User, E>> + Send;

    fn update_user(
        &self,
        id: UserId,
        params: UpdateUserParams,
    ) -> impl Future<Output = Result<User, E>> + Send;
}

impl<R, E> UserRepository<E> for &R
where
    R: UserRepository<E>,
    E: domain::Error,
{
    fn get_user(&self, id: UserId) -> impl Future<Output = Result<User, E>> + Send {
        R::get_user(self, id)
    }

    fn list_users(&self) -> impl Future<Output = Result<Vec<User>, E>> + Send {
        R::list_users(self)
    }

    fn create_user(
        &self,
        params: CreateUserParams,
    ) -> impl Future<Output = Result<User, E>> + Send {
        R::create_user(self, params)
    }

    fn update_user(
        &self,
        id: UserId,
        params: UpdateUserParams,
    ) -> impl Future<Output = Result<User, E>> + Send {
        R::update_user(self, id, params)
    }
}

impl<C, E> UserService<C, E> for super::Service
where
    C: UserRepository<E>,
    E: domain::Error,
{
    async fn get_user(&self, ctx: C, id: UserId) -> Result<User, E> {
        ctx.get_user(id).await
    }

    async fn list_users(&self, ctx: C) -> Result<Vec<User>, E> {
        ctx.list_users().await
    }

    async fn create_user(&self, ctx: C, params: CreateUserParams) -> Result<User, E> {
        ctx.create_user(params).await
    }

    async fn update_user(&self, ctx: C, id: UserId, params: UpdateUserParams) -> Result<User, E> {
        ctx.update_user(id, params).await
    }
}
