use domain::{CreateUserParams, UpdateUserParams, User, UserId, UserService};

// MARK: UserRepository

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

// MARK: impl for Service

impl<C, E> UserService<C, E> for super::Service
where
    C: UserRepository<E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_user(&self, _ctx: C, id: UserId) -> Result<User, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }

    #[tracing::instrument(skip_all)]
    async fn list_users(&self, _ctx: C) -> Result<Vec<User>, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }

    #[tracing::instrument(skip_all)]
    async fn create_user(&self, ctx: C, params: CreateUserParams) -> Result<User, E> {
        ctx.create_user(params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Created user");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_user(&self, _ctx: C, id: UserId, _params: UpdateUserParams) -> Result<User, E> {
        Err(E::unauthenticated("Unauthenticated access"))
    }
}

// MARK: impl for AuthenticatedService

impl<C, E> UserService<C, E> for super::AuthenticatedService
where
    C: UserRepository<E>,
    E: domain::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_user(&self, ctx: C, id: UserId) -> Result<User, E> {
        ctx.get_user(id).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Retrieved user");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn list_users(&self, ctx: C) -> Result<Vec<User>, E> {
        ctx.list_users().await.inspect(|us| {
            tracing::debug!(count = us.len(), "Listed users");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn create_user(&self, ctx: C, params: CreateUserParams) -> Result<User, E> {
        ctx.create_user(params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Created user");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_user(&self, ctx: C, id: UserId, params: UpdateUserParams) -> Result<User, E> {
        ctx.update_user(id, params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Updated user");
        })
    }
}
