use serde::{Deserialize, Serialize};

use domain::{CreateUserParams, UpdateUserParams, User, UserId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: domain::Timestamp,
    pub updated_at: domain::Timestamp,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        let User {
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
pub struct CreateUserRequest {
    pub name: String,
}

impl From<CreateUserRequest> for CreateUserParams {
    fn from(value: CreateUserRequest) -> Self {
        let CreateUserRequest { name } = value;
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UpdateUserRequest {
    pub name: String,
}

impl From<UpdateUserRequest> for UpdateUserParams {
    fn from(value: UpdateUserRequest) -> Self {
        let UpdateUserRequest { name } = value;
        Self { name }
    }
}

impl<T> crate::Service<T>
where
    T: crate::StateRequirements,
{
    pub(crate) async fn get_user(&self, user_id: uuid::Uuid) -> Result<UserResponse, crate::Error> {
        let user = self
            .0
            .get_user(UserId::new(user_id))
            .await
            .map_err(Into::into)?;
        Ok(user.into())
    }

    pub(crate) async fn list_users(&self) -> Result<Vec<UserResponse>, crate::Error> {
        let users = self.0.list_users().await.map_err(Into::into)?;
        let users: Vec<_> = users.into_iter().map(UserResponse::from).collect();
        Ok(users)
    }

    pub(crate) async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<UserResponse, crate::Error> {
        let user = self
            .0
            .create_user(request.into())
            .await
            .map_err(Into::into)?;
        Ok(user.into())
    }

    pub(crate) async fn update_user(
        &self,
        user_id: uuid::Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, crate::Error> {
        let user = self
            .0
            .update_user(UserId::new(user_id), request.into())
            .await
            .map_err(Into::into)?;
        Ok(user.into())
    }

    pub(crate) fn user_router(&self) -> axum::Router<Self> {
        use axum::Json;
        use axum::extract::{Path, State};
        use axum::routing::get;

        axum::Router::new()
            .route(
                "/users",
                get(async |State(s): State<Self>| s.list_users().await.map(Json))
                    .post(async |State(s): State<Self>, Json(r)| s.create_user(r).await.map(Json)),
            )
            .route(
                "/users/{id}",
                get(async |State(s): State<Self>, Path(id)| s.get_user(id).await.map(Json)).put(
                    async |State(s): State<Self>, Path(id), Json(r)| {
                        s.update_user(id, r).await.map(Json)
                    },
                ),
            )
    }
}
