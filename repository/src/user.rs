use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::FromRow,
)]
pub struct UserRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: domain::Timestamp,
    pub updated_at: domain::Timestamp,
}

impl From<UserRow> for domain::User {
    fn from(row: UserRow) -> Self {
        let UserRow {
            id,
            name,
            created_at,
            updated_at,
        } = row;
        domain::User {
            id: domain::UserId::new(id),
            name,
            created_at,
            updated_at,
        }
    }
}

impl<C, E> service::UserRepository<C, E> for crate::Repository
where
    C: crate::AsPgPool,
    E: crate::Error,
{
    async fn get_user(&self, ctx: C, id: domain::UserId) -> Result<domain::User, E> {
        let user = sqlx::query_file_as!(UserRow, "queries/get_user.sql", id.into_inner())
            .fetch_optional(ctx.as_pg_pool())
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while fetching user");
            })
            .context("Failed to fetch user from database")?
            .ok_or_else(|| E::not_found("User not found"))?;
        Ok(user.into())
    }

    async fn list_users(&self, ctx: C) -> Result<Vec<domain::User>, E> {
        let users = sqlx::query_file_as!(UserRow, "queries/list_users.sql")
            .fetch_all(ctx.as_pg_pool())
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while listing users");
            })
            .context("Failed to fetch users from database")?;
        Ok(users.into_iter().map(Into::into).collect())
    }

    async fn create_user(
        &self,
        ctx: C,
        params: domain::CreateUserParams,
    ) -> Result<domain::User, E> {
        let id = uuid::Uuid::now_v7();
        let domain::CreateUserParams { name } = params;
        let user = sqlx::query_file_as!(UserRow, "queries/create_user.sql", id, name)
            .fetch_one(ctx.as_pg_pool())
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while creating user");
            })
            .context("Failed to create user in database")?;
        Ok(user.into())
    }

    async fn update_user(
        &self,
        ctx: C,
        id: domain::UserId,
        params: domain::UpdateUserParams,
    ) -> Result<domain::User, E> {
        let domain::UpdateUserParams { name } = params;
        let user = sqlx::query_file_as!(UserRow, "queries/update_user.sql", id.into_inner(), name)
            .fetch_one(ctx.as_pg_pool())
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while updating user");
            })
            .context("Failed to update user in database")?;
        Ok(user.into())
    }
}
