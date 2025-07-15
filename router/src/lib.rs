use std::sync::Arc;

pub mod error;
mod group;
mod user;

pub use error::Error;

pub struct Service<T>(Arc<T>);

pub trait StateRequirements:
    domain::ProvideUserService<Error = Self::Err>
    + domain::ProvideGroupService<Error = Self::Err>
    + 'static
{
    type Err: domain::Error + Into<Error>;
}

impl<T, E> StateRequirements for T
where
    T: domain::ProvideUserService<Error = E> + domain::ProvideGroupService<Error = E> + 'static,
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

        let api = axum::Router::new()
            .merge(self.group_router())
            .merge(self.user_router());
        axum::Router::new()
            .route("/ping", get(async || "pong"))
            .nest("/api", api)
            .with_state(self)
    }
}
