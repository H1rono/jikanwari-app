pub struct AuthenticatedService<A> {
    pub service: A,
}

pub enum Rejection {
    TypedHeader(axum_extra::typed_header::TypedHeaderRejection),
    Error(crate::Error),
}

impl From<axum_extra::typed_header::TypedHeaderRejection> for Rejection {
    fn from(value: axum_extra::typed_header::TypedHeaderRejection) -> Self {
        Self::TypedHeader(value)
    }
}

impl From<crate::Error> for Rejection {
    fn from(value: crate::Error) -> Self {
        Self::Error(value)
    }
}

impl axum::response::IntoResponse for Rejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::TypedHeader(rejection) => rejection.into_response(),
            Self::Error(error) => error.into_response(),
        }
    }
}

impl<T, A> axum::extract::FromRequestParts<crate::Service<T>> for AuthenticatedService<A>
where
    T: crate::StateRequirements<Authn = A>,
    A: crate::AuthenticatedRequirements<Err = T::Err>,
{
    type Rejection = Rejection;

    #[tracing::instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &crate::Service<T>,
    ) -> Result<Self, Self::Rejection> {
        use axum_extra::TypedHeader;
        use headers::authorization::{Authorization, Bearer};
        // TODO: Implement proper authentication logic
        let bearer_token: TypedHeader<Authorization<Bearer>> =
            TypedHeader::from_request_parts(parts, state).await?;
        tracing::debug!("Extracted bearer token: {:?}", bearer_token);
        let user_id: uuid::Uuid = bearer_token.token().parse().map_err(|e| {
            let message = format!("Invalid token format: {e}");
            crate::Error::new(http::StatusCode::UNAUTHORIZED, message)
        })?;
        let service = state
            .0
            .make_authenticated(domain::UserId::new(user_id))
            .await
            .map_err(|e| e.into().replace_status(http::StatusCode::UNAUTHORIZED))?;
        Ok(AuthenticatedService { service })
    }
}
