use domain::{CreateUserParams, UpdateUserParams, User, UserId, UserService};

use crate::rbac::UserAccessControl;

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
    C: UserRepository<E> + UserAccessControl<E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_user(&self, ctx: C, id: UserId) -> Result<User, E> {
        ctx.judge_get_user(self.principal(), id)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "Anonymous access denied for user retrieval");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.get_user(id).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Retrieved user");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn list_users(&self, ctx: C) -> Result<Vec<User>, E> {
        ctx.judge_list_users(self.principal())
            .await?
            .allow_or_else(|| {
                tracing::debug!("Anonymous access denied for user listing");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.list_users().await.inspect(|us| {
            tracing::debug!(count = us.len(), "Listed users");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn create_user(&self, ctx: C, params: CreateUserParams) -> Result<User, E> {
        ctx.judge_create_user(self.principal(), &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!("Anonymous access denied for user creation");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.create_user(params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Created user");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_user(&self, ctx: C, id: UserId, params: UpdateUserParams) -> Result<User, E> {
        ctx.judge_update_user(self.principal(), id, &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "Anonymous access denied for user update");
                E::unauthenticated("Unauthenticated access")
            })?;
        ctx.update_user(id, params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Updated user");
        })
    }
}

// MARK: impl for AuthenticatedService

impl<C, E> UserService<C, E> for super::AuthenticatedService
where
    C: UserRepository<E> + UserAccessControl<E>,
    E: crate::Error,
{
    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn get_user(&self, ctx: C, id: UserId) -> Result<User, E> {
        ctx.judge_get_user(self.principal(), id)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "User access denied for user retrieval");
                E::forbidden("Access forbidden")
            })?;
        ctx.get_user(id).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Retrieved user");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn list_users(&self, ctx: C) -> Result<Vec<User>, E> {
        ctx.judge_list_users(self.principal())
            .await?
            .allow_or_else(|| {
                tracing::debug!("User access denied for user listing");
                E::forbidden("Access forbidden")
            })?;
        ctx.list_users().await.inspect(|us| {
            tracing::debug!(count = us.len(), "Listed users");
        })
    }

    #[tracing::instrument(skip_all)]
    async fn create_user(&self, ctx: C, params: CreateUserParams) -> Result<User, E> {
        ctx.judge_create_user(self.principal(), &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!("User access denied for user creation");
                E::forbidden("Access forbidden")
            })?;
        ctx.create_user(params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Created user");
        })
    }

    #[tracing::instrument(skip_all, fields(id = %id))]
    async fn update_user(&self, ctx: C, id: UserId, params: UpdateUserParams) -> Result<User, E> {
        ctx.judge_update_user(self.principal(), id, &params)
            .await?
            .allow_or_else(|| {
                tracing::debug!(id = %id, "User access denied for user update");
                E::forbidden("Access forbidden")
            })?;
        ctx.update_user(id, params).await.inspect(|u| {
            tracing::debug!(id = %u.id, "Updated user");
        })
    }
}
