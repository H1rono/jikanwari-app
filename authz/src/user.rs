use anyhow::Context;
use cedar_policy::EntityUid;

// MARK: UserEngine

#[derive(Debug, Clone)]
pub(crate) struct UserEngine {
    policies: cedar_policy::PolicySet,
    action_get: EntityUid,
    action_list: EntityUid,
    action_create: EntityUid,
    action_update: EntityUid,
    resource_create_user: EntityUid,
    resource_list_users: EntityUid,
}

impl UserEngine {
    pub(crate) const POLICIES: &str = include_str!("policies/user.cedar");
    pub(crate) const GET_ID: &str = "get-user";
    pub(crate) const LIST_ID: &str = "list-users";
    pub(crate) const CREATE_ID: &str = "create-user";
    pub(crate) const UPDATE_ID: &str = "update-user";
    pub(crate) const CREATE_USER_TYPE: &str = "CreateUser";
    pub(crate) const LIST_USERS_TYPE: &str = "ListUsers";

    pub(crate) fn new() -> anyhow::Result<Self> {
        use cedar_policy::EntityId;

        let policies = Self::POLICIES
            .parse()
            .context("Failed to parse user policies")?;
        let action = crate::Engine::action_type();
        let get = EntityId::new(Self::GET_ID);
        let list = EntityId::new(Self::LIST_ID);
        let create = EntityId::new(Self::CREATE_ID);
        let update = EntityId::new(Self::UPDATE_ID);
        let resource_create_user =
            EntityUid::from_type_name_and_id(Self::create_user_type()?, EntityId::new(""));
        let resource_list_users =
            EntityUid::from_type_name_and_id(Self::list_users_type()?, EntityId::new(""));
        Ok(Self {
            policies,
            action_get: EntityUid::from_type_name_and_id(action.clone(), get),
            action_list: EntityUid::from_type_name_and_id(action.clone(), list),
            action_create: EntityUid::from_type_name_and_id(action.clone(), create),
            action_update: EntityUid::from_type_name_and_id(action, update),
            resource_create_user,
            resource_list_users,
        })
    }

    fn create_user_type() -> anyhow::Result<cedar_policy::EntityTypeName> {
        Self::CREATE_USER_TYPE
            .parse()
            .context("Failed to parse create user type")
    }

    fn list_users_type() -> anyhow::Result<cedar_policy::EntityTypeName> {
        Self::LIST_USERS_TYPE
            .parse()
            .context("Failed to parse list user type")
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
        let engine = self.user();
        let action = engine.action_get.clone();
        let resource = self.encode_user_id(user_id)?;
        let context = cedar_policy::Context::empty();
        let request = self.make_request(by, action, resource, context)?;
        let policies = &engine.policies;
        let entities = cedar_policy::Entities::empty();
        let response = self
            .authorizer()
            .is_authorized(&request, policies, &entities);
        Ok(self.read_response(response))
    }

    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_list_users(
        &self,
        _ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        let engine = self.user();
        let action = engine.action_list.clone();
        let resource = engine.resource_list_users.clone();
        let context = cedar_policy::Context::empty();
        let request = self.make_request(by, action, resource, context)?;
        let policies = &engine.policies;
        let entities = cedar_policy::Entities::empty();
        let response = self
            .authorizer()
            .is_authorized(&request, policies, &entities);
        Ok(self.read_response(response))
    }

    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_create_user(
        &self,
        _ctx: C,
        by: service::Principal,
        params: &domain::CreateUserParams,
    ) -> Result<service::Judgement, E> {
        let engine = self.user();
        let action = engine.action_create.clone();
        let resource = engine.resource_create_user.clone();
        let context = cedar_policy::Context::empty();
        let request = self.make_request(by, action, resource, context)?;
        let policies = &engine.policies;
        let entities = cedar_policy::Entities::empty();
        let response = self
            .authorizer()
            .is_authorized(&request, policies, &entities);
        Ok(self.read_response(response))
    }

    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_update_user(
        &self,
        _ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> Result<service::Judgement, E> {
        use std::collections::{HashMap, HashSet};

        let engine = self.user();
        let action = engine.action_update.clone();
        let resource = self.encode_user_id(user_id)?;
        let context = cedar_policy::Context::empty();
        let request = self.make_request(by, action, resource, context)?;
        let policies = &engine.policies;
        let principal_entity = {
            let uid = self.principal_uid(by)?;
            let attr_id = match by {
                service::Principal::Anonymous => "anonymous".to_string(),
                service::Principal::User(u) => u.to_string(),
            };
            let attrs: HashMap<_, _> = [(
                "id".to_string(),
                cedar_policy::RestrictedExpression::new_string(attr_id),
            )]
            .into_iter()
            .collect();
            cedar_policy::Entity::new(uid, attrs, HashSet::new())
                .context("Failed to make entity of principal")?
        };
        let resource_entity = {
            let uid = self.encode_user_id(user_id)?;
            let attr_id = user_id.to_string();
            let attrs: HashMap<_, _> = [(
                "id".to_string(),
                cedar_policy::RestrictedExpression::new_string(attr_id),
            )]
            .into_iter()
            .collect();
            cedar_policy::Entity::new(uid, attrs, HashSet::new())
                .context("Failed to make entity of user")?
        };
        // TODO: provide schema
        let entities =
            cedar_policy::Entities::from_entities([principal_entity, resource_entity], None)
                .context("Failed to make entities of update user request")?;
        let response = self
            .authorizer()
            .is_authorized(&request, policies, &entities);
        Ok(self.read_response(response))
    }
}
