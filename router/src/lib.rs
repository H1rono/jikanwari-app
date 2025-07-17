use std::sync::Arc;

mod authn;
pub mod error;
mod group;
mod user;

pub use error::Error;

pub struct Service<T>(Arc<T>);

pub trait StateRequirements:
    domain::ProvideUserService<Error = Self::Err>
    + service::MakeAuthenticated<Self::Err, Authenticated = Self::Authn>
    + 'static
{
    type Authn: AuthenticatedRequirements<Err = Self::Err>;
    type Err: domain::Error + Into<Error>;
}

pub trait AuthenticatedRequirements:
    domain::ProvideUserService<Error = Self::Err>
    + domain::ProvideGroupService<Error = Self::Err>
    + 'static
{
    type Err: domain::Error + Into<Error>;
}

impl<T, A, E> StateRequirements for T
where
    T: domain::ProvideUserService<Error = E>
        + service::MakeAuthenticated<E, Authenticated = A>
        + 'static,
    E: domain::Error + Into<Error>,
    A: AuthenticatedRequirements<Err = E> + 'static,
{
    type Authn = A;
    type Err = E;
}

impl<A, E> AuthenticatedRequirements for A
where
    A: domain::ProvideUserService<Error = E> + domain::ProvideGroupService<Error = E> + 'static,
    E: domain::Error + Into<Error>,
{
    type Err = E;
}

impl<T> Clone for Service<T> {
    fn clone(&self) -> Self {
        Service(self.0.clone())
    }
}

impl<T> Service<T>
where
    T: StateRequirements,
{
    pub fn new(service: T) -> Self {
        Service(Arc::new(service))
    }

    pub fn into_router(self) -> axum::Router {
        use axum::routing::get;
        use tower_http::{
            ServiceBuilderExt,
            request_id::MakeRequestUuid,
            sensitive_headers::SetSensitiveHeadersLayer,
            trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
        };

        const SENSITIVE_HEADERS: [http::HeaderName; 3] = [
            http::header::AUTHORIZATION,
            http::header::COOKIE,
            http::header::SET_COOKIE,
        ];

        let api = axum::Router::new()
            .merge(self.group_router())
            .merge(self.user_router());
        let layer = tower::ServiceBuilder::new()
            .set_x_request_id(MakeRequestUuid)
            .layer(SetSensitiveHeadersLayer::from_shared(std::sync::Arc::new(
                SENSITIVE_HEADERS,
            )))
            .propagate_x_request_id()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true))
                    .on_response(DefaultOnResponse::default().include_headers(true)),
            );
        axum::Router::new()
            .route("/ping", get(async || "pong"))
            .nest("/api", api)
            .with_state(self)
            .layer(layer)
    }
}
