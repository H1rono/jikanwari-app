use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::FromRow,
)]
pub struct GroupRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: domain::Timestamp,
    pub updated_at: domain::Timestamp,
    pub members: Vec<uuid::Uuid>,
}

impl From<GroupRow> for domain::Group {
    fn from(row: GroupRow) -> Self {
        let GroupRow {
            id,
            name,
            created_at,
            updated_at,
            members,
        } = row;
        let members = members.into_iter().map(domain::UserId::new).collect();
        Self {
            id: domain::GroupId::new(id),
            name,
            created_at,
            updated_at,
            members,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::FromRow,
)]
pub struct GroupCoreRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: domain::Timestamp,
    pub updated_at: domain::Timestamp,
}

impl From<GroupCoreRow> for domain::GroupCore {
    fn from(row: GroupCoreRow) -> Self {
        let GroupCoreRow {
            id,
            name,
            created_at,
            updated_at,
        } = row;
        Self {
            id: domain::GroupId::new(id),
            name,
            created_at,
            updated_at,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::FromRow,
)]
pub struct GroupMemberRow {
    pub group_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

impl<C, E> service::GroupRepository<C, E> for crate::Repository
where
    C: crate::AsPgPool,
    E: crate::Error,
{
    async fn get_group(&self, ctx: C, id: domain::GroupId) -> Result<domain::Group, E> {
        let group = sqlx::query_file_as!(GroupRow, "queries/get_group.sql", id.into_inner())
            .fetch_one(ctx.as_pg_pool())
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while fetching group");
            })
            .context("Failed to fetch group")?;
        Ok(group.into())
    }

    async fn list_groups(&self, ctx: C) -> Result<Vec<domain::GroupCore>, E> {
        let groups = sqlx::query_file_as!(GroupCoreRow, "queries/list_groups.sql")
            .fetch_all(ctx.as_pg_pool())
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while listing groups");
            })
            .context("Failed to fetch groups")?;
        Ok(groups.into_iter().map(Into::into).collect())
    }

    async fn create_group(
        &self,
        ctx: C,
        params: domain::CreateGroupParams,
    ) -> Result<domain::Group, E> {
        // TODO: transaction
        let mut conn = ctx
            .as_pg_pool()
            .acquire()
            .await
            .context("Failed to acquire connection")?;
        let id = uuid::Uuid::now_v7();
        let domain::CreateGroupParams { name, members } = params;
        let members: Vec<_> = members.into_iter().map(|id| id.into_inner()).collect();
        let group_core =
            sqlx::query_file_as!(GroupCoreRow, "queries/create_group_core.sql", id, name)
                .fetch_one(&mut *conn)
                .await
                .inspect_err(|e| {
                    tracing::error!(error = %e, "Postgres error while creating group core");
                })
                .context("Failed to create group core")?;
        let group_members = sqlx::query_file_as!(
            GroupMemberRow,
            "queries/create_group_members.sql",
            id,
            &members
        )
        .fetch_all(&mut *conn)
        .await
        .inspect_err(|e| {
            tracing::error!(error = %e, "Postgres error while creating group members");
        })
        .context("Failed to create group members")?;
        let GroupCoreRow {
            id,
            name,
            created_at,
            updated_at,
        } = group_core;
        let members = group_members
            .into_iter()
            .map(|m| domain::UserId::new(m.user_id))
            .collect();
        Ok(domain::Group {
            id: domain::GroupId::new(id),
            name,
            created_at,
            updated_at,
            members,
        })
    }

    async fn update_group(
        &self,
        ctx: C,
        id: domain::GroupId,
        params: domain::UpdateGroupParams,
    ) -> Result<domain::Group, E> {
        let domain::UpdateGroupParams { name } = params;
        let group =
            sqlx::query_file_as!(GroupRow, "queries/update_group.sql", id.into_inner(), name)
                .fetch_one(ctx.as_pg_pool())
                .await
                .inspect_err(|e| {
                    tracing::error!(error = %e, "Postgres error while updating group");
                })
                .context("Failed to update group")?;
        Ok(group.into())
    }

    async fn update_group_members(
        &self,
        ctx: C,
        id: domain::GroupId,
        members: &[domain::UserId],
    ) -> Result<domain::Group, E> {
        #[derive(sqlx::FromRow)]
        struct Check {
            group_count: i64,
            member_count: i64,
        }

        self.within_tx(ctx.as_pg_pool(), async |conn| {
            let members: Vec<_> = members.iter().map(|id| id.into_inner()).collect();
            let check = sqlx::query_file_as!(
                Check,
                "queries/update_group_members.0.sql",
                id.into_inner(),
                &members
            )
            .fetch_one(&mut *conn)
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while checking group members");
            })
            .context("Failed to check group members")?;
            if check.group_count == 0 {
                return Err(E::not_found("Group not found"));
            }
            if check.member_count != members.len() as i64 {
                return Err(E::not_found("Some members not found"));
            }

            sqlx::query_file!("queries/update_group_members.1.sql", id.into_inner())
                .execute(&mut *conn)
                .await
                .inspect_err(|e| {
                    tracing::error!(
                        error = %e,
                        "Postgres error while deleting existing group members",
                    );
                })
                .context("Failed to delete existing group members")?;

            let _ = sqlx::query_file_as!(
                GroupMemberRow,
                "queries/update_group_members.2.sql",
                id.into_inner(),
                &members
            )
            .fetch_all(&mut *conn)
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "Postgres error while inserting new group members");
            })
            .context("Failed to insert new group members")?;

            let group = sqlx::query_file_as!(GroupRow, "queries/get_group.sql", id.into_inner())
                .fetch_one(&mut *conn)
                .await
                .inspect_err(|e| {
                    tracing::error!(error = %e, "Postgres error while fetching updated group");
                })
                .context("Failed to fetch updated group")?;
            Ok(group.into())
        })
        .await
    }
}
