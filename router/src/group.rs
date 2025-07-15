use serde::{Deserialize, Serialize};

use domain::{CreateGroupParams, Group, GroupCore, GroupId, UpdateGroupParams, UserId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct GroupResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: domain::Timestamp,
    pub updated_at: domain::Timestamp,
    pub members: Vec<uuid::Uuid>,
}

impl From<Group> for GroupResponse {
    fn from(value: Group) -> Self {
        let Group {
            id,
            name,
            created_at,
            updated_at,
            members,
        } = value;
        let members: Vec<_> = members.into_iter().map(UserId::into_inner).collect();
        Self {
            id: id.into_inner(),
            name,
            created_at,
            updated_at,
            members,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct GroupCoreResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: domain::Timestamp,
    pub updated_at: domain::Timestamp,
}

impl From<GroupCore> for GroupCoreResponse {
    fn from(value: GroupCore) -> Self {
        let GroupCore {
            id,
            name,
            created_at,
            updated_at,
        } = value;
        Self {
            id: id.into_inner(),
            name,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub members: Vec<uuid::Uuid>,
}

impl From<CreateGroupRequest> for CreateGroupParams {
    fn from(value: CreateGroupRequest) -> Self {
        let CreateGroupRequest { name, members } = value;
        let members: Vec<_> = members.into_iter().map(UserId::new).collect();
        Self { name, members }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UpdateGroupRequest {
    pub name: String,
}

impl From<UpdateGroupRequest> for UpdateGroupParams {
    fn from(value: UpdateGroupRequest) -> Self {
        let UpdateGroupRequest { name } = value;
        Self { name }
    }
}

impl<T> crate::Service<T>
where
    T: crate::StateRequirements,
{
    pub(crate) async fn get_group(
        &self,
        group_id: uuid::Uuid,
    ) -> Result<GroupResponse, crate::Error> {
        let group = self
            .0
            .get_group(GroupId::new(group_id))
            .await
            .map_err(Into::into)?;
        Ok(group.into())
    }

    pub(crate) async fn list_groups(&self) -> Result<Vec<GroupCoreResponse>, crate::Error> {
        let groups = self.0.list_groups().await.map_err(Into::into)?;
        let groups: Vec<_> = groups.into_iter().map(GroupCoreResponse::from).collect();
        Ok(groups)
    }

    pub(crate) async fn create_group(
        &self,
        request: CreateGroupRequest,
    ) -> Result<GroupResponse, crate::Error> {
        let group = self
            .0
            .create_group(request.into())
            .await
            .map_err(Into::into)?;
        Ok(group.into())
    }

    pub(crate) async fn update_group(
        &self,
        group_id: uuid::Uuid,
        request: UpdateGroupRequest,
    ) -> Result<GroupResponse, crate::Error> {
        let group = self
            .0
            .update_group(GroupId::new(group_id), request.into())
            .await
            .map_err(Into::into)?;
        Ok(group.into())
    }

    pub(crate) async fn update_group_members(
        &self,
        group_id: uuid::Uuid,
        members: Vec<uuid::Uuid>,
    ) -> Result<GroupResponse, crate::Error> {
        let members: Vec<_> = members.into_iter().map(UserId::new).collect();
        let group = self
            .0
            .update_group_members(GroupId::new(group_id), &members)
            .await
            .map_err(Into::into)?;
        Ok(group.into())
    }

    pub(crate) fn group_router(&self) -> axum::Router<Self> {
        use axum::Json;
        use axum::extract::{Path, State};
        use axum::routing::{get, put};

        axum::Router::new()
            .route(
                "/groups",
                get(async |State(s): State<Self>| s.list_groups().await.map(Json))
                    .post(async |State(s): State<Self>, Json(r)| s.create_group(r).await.map(Json)),
            )
            .route(
                "/groups/{id}",
                get(async |State(s): State<Self>, Path(id)| s.get_group(id).await.map(Json)).put(
                    async |State(s): State<Self>, Path(id), Json(r)| {
                        s.update_group(id, r).await.map(Json)
                    },
                ),
            )
            .route(
                "/groups/{id}/members",
                put(async |State(s): State<Self>, Path(id), Json(r)| {
                    s.update_group_members(id, r).await.map(Json)
                }),
            )
    }
}
