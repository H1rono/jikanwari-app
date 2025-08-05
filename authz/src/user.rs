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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Request {
    GetUser(domain::UserId),
    ListUsers,
    CreateUser,
    UpdateUser(domain::UserId),
}

impl crate::Engine {
    pub(crate) async fn process_user_request<E: crate::Error>(
        &self,
        by: service::Principal,
        request: Request,
    ) -> Result<service::Judgement, E> {
        use Request::{CreateUser, GetUser, ListUsers, UpdateUser};

        let engine = self.user();
        let action = match request {
            GetUser(_) => engine.action_get.clone(),
            ListUsers => engine.action_list.clone(),
            CreateUser => engine.action_create.clone(),
            UpdateUser(_) => engine.action_update.clone(),
        };
        let resource = match request {
            GetUser(user_id) => self.encode_user_id(user_id)?,
            ListUsers => engine.resource_list_users.clone(),
            CreateUser => engine.resource_create_user.clone(),
            UpdateUser(user_id) => self.encode_user_id(user_id)?,
        };
        let context = cedar_policy::Context::empty();
        let entities = match request {
            GetUser(_) | ListUsers | CreateUser => cedar_policy::Entities::empty(),
            UpdateUser(user_id) => {
                let principal_entity = self.encode_principal_entity(by, std::iter::empty())?;
                let resource_entity = self.encode_user_entity(user_id, std::iter::empty())?;
                cedar_policy::Entities::from_entities([principal_entity, resource_entity], None)
                    .context("Failed to make entities of user request")?
            }
        };
        let request = self.make_request(by, action, resource, context)?;
        let policies = &engine.policies;
        let response = self
            .authorizer()
            .is_authorized(&request, policies, &entities);
        Ok(self.read_response(response))
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
        let r = Request::GetUser(user_id);
        self.process_user_request::<E>(by, r).await
    }

    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_list_users(
        &self,
        _ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        let r = Request::ListUsers;
        self.process_user_request::<E>(by, r).await
    }

    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_create_user(
        &self,
        _ctx: C,
        by: service::Principal,
        params: &domain::CreateUserParams,
    ) -> Result<service::Judgement, E> {
        let r = Request::CreateUser;
        self.process_user_request::<E>(by, r).await
    }

    #[tracing::instrument(skip(self, _ctx), ret(level = "debug"))]
    async fn judge_update_user(
        &self,
        _ctx: C,
        by: service::Principal,
        user_id: domain::UserId,
        params: &domain::UpdateUserParams,
    ) -> Result<service::Judgement, E> {
        let r = Request::UpdateUser(user_id);
        self.process_user_request::<E>(by, r).await
    }
}
