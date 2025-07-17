mod group;
mod user;

pub trait Error: domain::Error {
    fn unauthenticated(message: &str) -> Self;
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Service(());

impl Service {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

pub trait MakeAuthenticated<E>: Send + Sync {
    type Authenticated: Send + Sync;

    fn make_authenticated(
        &self,
        user_id: domain::UserId,
    ) -> impl Future<Output = Result<Self::Authenticated, E>> + Send;
}

#[expect(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedService {
    service: Service,
    user_id: domain::UserId,
}

impl Service {
    pub fn authenticated(&self, user_id: domain::UserId) -> AuthenticatedService {
        AuthenticatedService {
            service: *self,
            user_id,
        }
    }
}

pub use group::GroupRepository;
pub use user::UserRepository;
