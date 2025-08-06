mod group;
mod user;

pub use group::{GroupEntityRepository, ProvideGroupEntityRepository};

#[derive(Debug, Clone)]
pub struct Engine(std::sync::Arc<EngineInner>);

#[derive(Debug, Clone)]
struct EngineInner {
    authorizer: cedar_policy::Authorizer,
    user: user::UserEngine,
    group: group::GroupEngine,
    user_type: cedar_policy::EntityTypeName,
    group_type: cedar_policy::EntityTypeName,
    anonymous_id: cedar_policy::EntityId,
}

impl Engine {
    const USER_TYPE: &str = "User";
    const GROUP_TYPE: &str = "Group";
    const ANONYMOUS_ID: &str = "anonymous";
    const ACTION_TYPE: &str = "Action";

    pub fn new() -> anyhow::Result<Self> {
        use anyhow::Context;

        let authorizer = cedar_policy::Authorizer::new();
        let user = user::UserEngine::new()?;
        let group = group::GroupEngine::new()?;
        let user_type = Self::USER_TYPE
            .parse()
            .context("Failed to parse user type")?;
        let group_type = Self::GROUP_TYPE
            .parse()
            .context("Failed to parse group type")?;
        let anonymous_id = cedar_policy::EntityId::new(Self::ANONYMOUS_ID);
        let inner = EngineInner {
            authorizer,
            user,
            group,
            user_type,
            group_type,
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

    fn group(&self) -> &group::GroupEngine {
        &self.0.group
    }

    fn user_type(&self) -> &cedar_policy::EntityTypeName {
        &self.0.user_type
    }

    fn group_type(&self) -> &cedar_policy::EntityTypeName {
        &self.0.group_type
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

    fn encode_group_id(&self, id: domain::GroupId) -> anyhow::Result<cedar_policy::EntityUid> {
        use anyhow::Context;

        let ty = self.group_type().clone();
        let id = id
            .to_string()
            .parse()
            .context("Failed to parse GroupId as entity ID")?;
        Ok(cedar_policy::EntityUid::from_type_name_and_id(ty, id))
    }

    /// principal -> `User` entity
    fn encode_principal_entity(
        &self,
        p: service::Principal,
        groups: impl IntoIterator<Item = domain::GroupId>,
    ) -> anyhow::Result<cedar_policy::Entity> {
        use std::collections::{HashMap, HashSet};

        use anyhow::Context;

        let uid = self.principal_uid(p)?;
        let attr_id = match p {
            service::Principal::Anonymous => "anonymous".to_string(),
            service::Principal::User(u) => u.to_string(),
        };
        let attrs: HashMap<_, _> = [(
            "id".to_string(),
            cedar_policy::RestrictedExpression::new_string(attr_id),
        )]
        .into_iter()
        .collect();
        let groups: HashSet<_> = groups
            .into_iter()
            .map(|g| self.encode_group_id(g))
            .collect::<anyhow::Result<_>>()?;
        cedar_policy::Entity::new(uid, attrs, groups).context("Failed to make entity of principal")
    }

    /// user -> `User` entity
    fn encode_user_entity(
        &self,
        user_id: domain::UserId,
        groups: impl IntoIterator<Item = domain::GroupId>,
    ) -> anyhow::Result<cedar_policy::Entity> {
        use std::collections::{HashMap, HashSet};

        use anyhow::Context;

        let uid = self.encode_user_id(user_id)?;
        let attr_id = user_id.to_string();
        let attrs: HashMap<_, _> = [(
            "id".to_string(),
            cedar_policy::RestrictedExpression::new_string(attr_id),
        )]
        .into_iter()
        .collect();
        let groups: HashSet<_> = groups
            .into_iter()
            .map(|g| self.encode_group_id(g))
            .collect::<anyhow::Result<_>>()?;
        cedar_policy::Entity::new(uid, attrs, groups).context("Failed to make entity of user")
    }

    fn encode_group_members(
        &self,
        members: &[domain::UserId],
    ) -> anyhow::Result<cedar_policy::RestrictedExpression> {
        use anyhow::Context;
        use cedar_policy::RestrictedExpression;

        let members = members
            .iter()
            .map(|m| {
                let id = RestrictedExpression::new_string(m.to_string());
                RestrictedExpression::new_record([("id".to_string(), id)])
            })
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to make cedar expression of group members")?;
        Ok(RestrictedExpression::new_set(members))
    }

    /// group -> `Group` entity
    fn encode_group_entity(
        &self,
        id: domain::GroupId,
        members: &[domain::UserId],
    ) -> anyhow::Result<cedar_policy::Entity> {
        use std::collections::{HashMap, HashSet};

        use anyhow::Context;

        let uid = self.encode_group_id(id)?;
        let id = cedar_policy::RestrictedExpression::new_string(id.to_string());
        let members = self.encode_group_members(members)?;
        let attrs: HashMap<_, _> = [("id".to_string(), id), ("members".to_string(), members)]
            .into_iter()
            .collect();
        cedar_policy::Entity::new(uid, attrs, HashSet::new())
            .context("Failed to make entity of group")
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

// MARK: trait Error

pub trait Error: domain::Error + From<anyhow::Error> {}
