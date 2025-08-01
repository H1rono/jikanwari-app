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
        let anonymous_id = cedar_policy::EntityId::new(Self::ANONYMOUS_ID);
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

    fn principal_uid(&self, p: service::Principal) -> anyhow::Result<cedar_policy::EntityUid> {
        use anyhow::Context;

        let uid = match p {
            service::Principal::Anonymous => self.principal_anonymous(),
            service::Principal::User(id) => {
                let p = format!(r#"User::"{id}""#);
                p.parse().context("Failed to parse user principal")?
            }
        };
        Ok(uid)
    }

    fn encode_user_id(&self, id: domain::UserId) -> anyhow::Result<cedar_policy::EntityUid> {
        use anyhow::Context;

        let ty = self.user_type().clone();
        let id = id
            .to_string()
            .parse()
            .context("Failed to parse UserId as entity ID")?;
        Ok(cedar_policy::EntityUid::from_type_name_and_id(ty, id))
    }

    fn make_request(
        &self,
        by: service::Principal,
        action: cedar_policy::EntityUid,
        resource: cedar_policy::EntityUid,
        context: cedar_policy::Context,
    ) -> anyhow::Result<cedar_policy::Request> {
        use anyhow::Context;

        let p = self.principal_uid(by)?;
        // TODO: provide schema
        let request = cedar_policy::Request::new(p, action, resource, context, None)
            .context("Failed to make cedar request")?;
        Ok(request)
    }

    fn read_response(&self, response: cedar_policy::Response) -> service::Judgement {
        tracing::debug!(?response);
        for e in response.diagnostics().errors() {
            tracing::error!(error = %e, "Cedar policy error");
        }
        match response.decision() {
            cedar_policy::Decision::Allow => service::Judgement::Allow,
            cedar_policy::Decision::Deny => service::Judgement::Deny,
        }
    }
}

pub trait Error: domain::Error + From<anyhow::Error> {}
