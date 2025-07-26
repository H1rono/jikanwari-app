mod group;
mod user;

#[derive(Debug, Clone)]
pub struct Engine(std::sync::Arc<EngineInner>);

#[derive(Debug, Clone)]
struct EngineInner {
    authorizer: cedar_policy::Authorizer,
    user: user::UserEngine,
}

impl Engine {
    pub fn new() -> anyhow::Result<Self> {
        let authorizer = cedar_policy::Authorizer::new();
        let user = user::UserEngine::new()?;
        let inner = EngineInner { authorizer, user };
        Ok(Self(std::sync::Arc::new(inner)))
    }

    fn authorizer(&self) -> &cedar_policy::Authorizer {
        &self.0.authorizer
    }

    fn user(&self) -> &user::UserEngine {
        &self.0.user
    }
}

pub trait Error: domain::Error + From<anyhow::Error> {}
