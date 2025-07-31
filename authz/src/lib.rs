mod group;
mod user;

#[derive(Debug, Clone)]
pub struct Engine(std::sync::Arc<EngineInner>);

#[derive(Debug, Clone)]
struct EngineInner {
    authorizer: cedar_policy::Authorizer,
    user: user::UserEngine,
    user_type: cedar_policy::EntityTypeName,
    anonymous_id: cedar_policy::EntityId,
}

impl Engine {
    const USER_TYPE: &str = "User";
    const ANONYMOUS_ID: &str = "anonymous";
    const ACTION_TYPE: &str = "Action";

    pub fn new() -> anyhow::Result<Self> {
        use anyhow::Context;

        let authorizer = cedar_policy::Authorizer::new();
        let user = user::UserEngine::new()?;
        let user_type = Self::USER_TYPE
            .parse()
            .context("Failed to parse user type")?;
        let anonymous_id = Self::ANONYMOUS_ID
            .parse()
            .context("Failed to parse anonymous user ID")?;
        let inner = EngineInner {
            authorizer,
            user,
            user_type,
            anonymous_id,
        };
        Ok(Self(std::sync::Arc::new(inner)))
    }

    fn authorizer(&self) -> &cedar_policy::Authorizer {
        &self.0.authorizer
    }

    fn user(&self) -> &user::UserEngine {
        &self.0.user
    }

    fn user_type(&self) -> &cedar_policy::EntityTypeName {
        &self.0.user_type
    }

    fn anonymous_id(&self) -> &cedar_policy::EntityId {
        &self.0.anonymous_id
    }

    fn principal_anonymous(&self) -> cedar_policy::EntityUid {
        cedar_policy::EntityUid::from_type_name_and_id(
            self.user_type().clone(),
            self.anonymous_id().clone(),
        )
    }

    fn action_type() -> cedar_policy::EntityTypeName {
        Self::ACTION_TYPE.parse().unwrap()
    }
}

pub trait Error: domain::Error + From<anyhow::Error> {}
