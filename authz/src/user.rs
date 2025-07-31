use anyhow::Context;
use cedar_policy::EntityUid;

use service::Principal;

// MARK: UserEngine

#[derive(Debug, Clone)]
pub(crate) struct UserEngine {
    policies: cedar_policy::PolicySet,
    action_get: EntityUid,
    action_list: EntityUid,
    action_create: EntityUid,
    action_update: EntityUid,
}

impl UserEngine {
    pub(crate) const POLICIES: &str = include_str!("policies/user.cedar");
    pub(crate) const GET_ID: &str = "get-user";
    pub(crate) const LIST_ID: &str = "list-users";
    pub(crate) const CREATE_ID: &str = "create-user";
    pub(crate) const UPDATE_ID: &str = "update-user";

    pub(crate) fn new() -> anyhow::Result<Self> {
        let policies = Self::POLICIES
            .parse()
            .context("Failed to parse user policies")?;
        let action = crate::Engine::action_type();
        let get = Self::GET_ID
            .parse()
            .context("Failed to parse create user action ID")?;
        let list = Self::LIST_ID
            .parse()
            .context("Failed to parse list users action ID")?;
        let create = Self::CREATE_ID
            .parse()
            .context("Failed to parse create user action ID")?;
        let update = Self::UPDATE_ID
            .parse()
            .context("Failed to parse update user action ID")?;
        Ok(Self {
            policies,
            action_get: EntityUid::from_type_name_and_id(action.clone(), get),
            action_list: EntityUid::from_type_name_and_id(action.clone(), list),
            action_create: EntityUid::from_type_name_and_id(action.clone(), create),
            action_update: EntityUid::from_type_name_and_id(action, update),
        })
    }
}

// MARK: UserAccessControl for Engine

impl<C, E> service::UserAccessControl<C, E> for crate::Engine
where
    C: Send + Sync,
    E: crate::Error,
{
    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_get_user(
        &self,
        _ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
    ) -> Result<service::Judgement, E> {
        let authorizer = self.authorizer();
        let engine = self.user();
        let principal = match by {
            Principal::Anonymous => self.principal_anonymous(),
            Principal::User(id) => {
                let p = format!(r#"User::"{id}""#);
                p.parse().context("Failed to parse user principal")?
            }
        };
        let action = engine.action_get.clone();
        let resource = format!(r#"User::"{user_id}""#)
            .parse()
            .context("Failed to parse user resource")?;
        let context = cedar_policy::Context::empty();
        let request = cedar_policy::Request::new(principal, action, resource, context, None)
            .context("Failed to create cedar request")?;
        let policies = &engine.policies;
        let entities = cedar_policy::Entities::empty();
        let decision = authorizer.is_authorized(&request, policies, &entities);
        tracing::debug!(?decision);
        match decision.decision() {
            cedar_policy::Decision::Allow => Ok(service::Judgement::Allow),
            cedar_policy::Decision::Deny => Ok(service::Judgement::Deny),
        }
    }

    async fn judge_list_users(
        &self,
        ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }

    async fn judge_create_user(
        &self,
        ctx: C,
        by: service::Principal,
        params: &domain::CreateUserParams,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }

    async fn judge_update_user(
        &self,
        ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }
}
