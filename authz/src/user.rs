use anyhow::Context;
use cedar_policy::EntityUid;

use service::Principal;

// MARK: UserEngine

#[derive(Debug, Clone)]
pub(crate) struct UserEngine {
    policies: cedar_policy::PolicySet,
    principal_anonymous: EntityUid,
    action_get: EntityUid,
    action_list: EntityUid,
    action_create: EntityUid,
    action_update: EntityUid,
}

impl UserEngine {
    pub(crate) const POLICIES: &str = include_str!("policies/user.cedar");
    pub(crate) const PRINCIPAL_ANONYMOUS: &str = r#"User::"anonymous""#;
    pub(crate) const ACTION_GET: &str = r#"UserAction::"get""#;
    pub(crate) const ACTION_LIST: &str = r#"UserAction::"list""#;
    pub(crate) const ACTION_CREATE: &str = r#"UserAction::"create""#;
    pub(crate) const ACTION_UPDATE: &str = r#"UserAction::"update""#;

    pub(crate) fn new() -> anyhow::Result<Self> {
        let policies = Self::POLICIES
            .parse()
            .context("Failed to parse user policies")?;
        let principal_anonymous = Self::PRINCIPAL_ANONYMOUS
            .parse()
            .context("Failed to parse user principal 'anonymous'")?;
        let action_get = Self::ACTION_GET
            .parse()
            .context("Failed to parse user action 'get'")?;
        let action_list = Self::ACTION_LIST
            .parse()
            .context("Failed to parse user action 'list'")?;
        let action_create = Self::ACTION_CREATE
            .parse()
            .context("Failed to parse user action 'create'")?;
        let action_update = Self::ACTION_UPDATE
            .parse()
            .context("Failed to parse user action 'update'")?;
        Ok(Self {
            policies,
            principal_anonymous,
            action_get,
            action_list,
            action_create,
            action_update,
        })
    }
}

// MARK: UserAccessControl for Engine

impl<C, E> service::UserAccessControl<C, E> for crate::Engine
where
    C: Send + Sync,
    E: crate::Error,
{
    #[tracing::instrument(skip(self, ctx, by, user_id))]
    async fn judge_get_user(
        &self,
        ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
    ) -> Result<service::Judgement, E> {
        let authorizer = self.authorizer();
        let engine = self.user();
        let principal = match by {
            Principal::Anonymous => engine.principal_anonymous.clone(),
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
