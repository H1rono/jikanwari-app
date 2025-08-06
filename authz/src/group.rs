use anyhow::Context;
use cedar_policy::EntityUid;

// MARK: GroupEngine

#[derive(Debug, Clone)]
pub(crate) struct GroupEngine {
    policies: cedar_policy::PolicySet,
    action_get: EntityUid,
    action_list: EntityUid,
    action_create: EntityUid,
    action_update: EntityUid,
    action_update_members: EntityUid,
    resource_create_group: EntityUid,
    resource_list_groups: EntityUid,
}

impl GroupEngine {
    pub(crate) const POLICIES: &str = include_str!("policies/group.cedar");
    pub(crate) const GET_ID: &str = "get-group";
    pub(crate) const LIST_ID: &str = "list-groups";
    pub(crate) const CREATE_ID: &str = "create-group";
    pub(crate) const UPDATE_ID: &str = "update-group";
    pub(crate) const UPDATE_MEMBERS_ID: &str = "update-group-members";
    pub(crate) const CREATE_GROUP_TYPE: &str = "CreateGroup";
    pub(crate) const LIST_GROUPS_TYPE: &str = "ListGroups";

    pub(crate) fn new() -> anyhow::Result<Self> {
        use cedar_policy::EntityId;

        let policies = Self::POLICIES
            .parse()
            .context("Failed to parse group policies")?;
        let action = crate::Engine::action_type();
        let get = EntityId::new(Self::GET_ID);
        let list = EntityId::new(Self::LIST_ID);
        let create = EntityId::new(Self::CREATE_ID);
        let update = EntityId::new(Self::UPDATE_ID);
        let update_members = EntityId::new(Self::UPDATE_MEMBERS_ID);
        let resource_create_group =
            EntityUid::from_type_name_and_id(Self::create_group_type()?, EntityId::new(""));
        let resource_list_groups =
            EntityUid::from_type_name_and_id(Self::list_groups_type()?, EntityId::new(""));
        Ok(Self {
            policies,
            action_get: EntityUid::from_type_name_and_id(action.clone(), get),
            action_list: EntityUid::from_type_name_and_id(action.clone(), list),
            action_create: EntityUid::from_type_name_and_id(action.clone(), create),
            action_update: EntityUid::from_type_name_and_id(action.clone(), update),
            action_update_members: EntityUid::from_type_name_and_id(action, update_members),
            resource_create_group,
            resource_list_groups,
        })
    }

    fn create_group_type() -> anyhow::Result<cedar_policy::EntityTypeName> {
        Self::CREATE_GROUP_TYPE
            .parse()
            .context("Failed to parse create group type")
    }

    fn list_groups_type() -> anyhow::Result<cedar_policy::EntityTypeName> {
        Self::LIST_GROUPS_TYPE
            .parse()
            .context("Failed to parse list group type")
    }
}

// MARK: GroupEntityRepository

pub trait GroupEntityRepository<Context, E>: Send + Sync {
    fn get_group_members(
        &self,
        ctx: Context,
        id: domain::GroupId,
    ) -> impl Future<Output = Result<Vec<domain::UserId>, E>> + Send;
}

impl<R, C, E> GroupEntityRepository<C, E> for &R
where
    R: GroupEntityRepository<C, E>,
    C: Send,
{
    async fn get_group_members(
        &self,
        ctx: C,
        id: domain::GroupId,
    ) -> Result<Vec<domain::UserId>, E> {
        R::get_group_members(self, ctx, id).await
    }
}

pub trait ProvideGroupEntityRepository: Send + Sync {
    type Context<'a>
    where
        Self: 'a;
    type GroupEntityRepository<'a>: GroupEntityRepository<Self::Context<'a>, Self::Error>
    where
        Self: 'a;
    type Error;

    fn context(&self) -> Self::Context<'_>;
    fn group_entity_repository(&self) -> &Self::GroupEntityRepository<'_>;

    fn get_group_members(
        &self,
        id: domain::GroupId,
    ) -> impl Future<Output = Result<Vec<domain::UserId>, Self::Error>> + Send {
        let ctx = self.context();
        self.group_entity_repository().get_group_members(ctx, id)
    }
}

impl<R> ProvideGroupEntityRepository for &R
where
    R: ProvideGroupEntityRepository,
{
    type Context<'a>
        = R::Context<'a>
    where
        Self: 'a;
    type Error = R::Error;
    type GroupEntityRepository<'a>
        = R::GroupEntityRepository<'a>
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        R::context(self)
    }
    fn group_entity_repository(&self) -> &Self::GroupEntityRepository<'_> {
        R::group_entity_repository(self)
    }
}

// MARK: Request

#[derive(Debug, Clone)]
pub(crate) enum Request<'a> {
    GetGroup(domain::GroupId),
    ListGroups,
    CreateGroup {
        members: &'a [domain::UserId],
    },
    UpdateGroup(domain::GroupId),
    UpdateGroupMembers {
        id: domain::GroupId,
        members: &'a [domain::UserId],
    },
}

impl crate::Engine {
    fn encode_create_group_entity(
        &self,
        members: &[domain::UserId],
    ) -> anyhow::Result<cedar_policy::Entity> {
        use std::collections::{HashMap, HashSet};

        let uid = self.group().resource_create_group.clone();
        let members = self.encode_group_members(members)?;
        let attrs: HashMap<_, _> = [("members".to_string(), members)].into_iter().collect();
        cedar_policy::Entity::new(uid, attrs, HashSet::new())
            .context("Failed to make entity of create-group")
    }

    pub(crate) async fn process_group_request<E: crate::Error>(
        &self,
        by: service::Principal,
        repo: impl ProvideGroupEntityRepository<Error = E>,
        request: Request<'_>,
    ) -> Result<service::Judgement, E> {
        use Request::{CreateGroup, GetGroup, ListGroups, UpdateGroup, UpdateGroupMembers};

        let engine = self.group();
        let (action, resource, entities, policies) = match request {
            GetGroup(id) => {
                let action = engine.action_get.clone();
                let resource = self.encode_group_id(id)?;
                let entities = cedar_policy::Entities::empty();
                (action, resource, entities, engine.policies.clone())
            }
            ListGroups => {
                let action = engine.action_list.clone();
                let resource = engine.resource_list_groups.clone();
                let entities = cedar_policy::Entities::empty();
                (action, resource, entities, engine.policies.clone())
            }
            CreateGroup { members } => {
                let action = engine.action_create.clone();
                let resource = engine.resource_create_group.clone();
                let entities = {
                    let principal = self.encode_principal_entity(by, std::iter::empty())?;
                    let create_group = self.encode_create_group_entity(members)?;
                    cedar_policy::Entities::from_entities([principal, create_group], None)
                        .context("Failed to make cedar entities")?
                };
                (action, resource, entities, engine.policies.clone())
            }
            UpdateGroup(id) => {
                let members = repo.get_group_members(id).await?;
                let action = engine.action_update.clone();
                let resource = self.encode_group_id(id)?;
                let entities = {
                    let principal = match by {
                        service::Principal::User(user_id) if members.contains(&user_id) => {
                            self.encode_principal_entity(by, [id])?
                        }
                        service::Principal::User(_) | service::Principal::Anonymous => {
                            self.encode_principal_entity(by, std::iter::empty())?
                        }
                    };
                    let update_group = self.encode_group_entity(id, &members)?;
                    cedar_policy::Entities::from_entities([principal, update_group], None)
                        .context("Failed to make cedar entities")?
                };
                let policies = {
                    let mut p = engine.policies.clone();
                    let template = engine
                        .policies
                        .templates()
                        .find(|t| {
                            t.annotation("id")
                                .is_some_and(|i| i == "permit-update-group")
                        })
                        .context(r#"Policy template @id("permit-update-group") not found"#)?;
                    let template_id = template.id().clone();
                    let policy_id =
                        cedar_policy::PolicyId::new(format!("permit-update-group-{id}"));
                    let env: std::collections::HashMap<_, _> =
                        [(cedar_policy::SlotId::principal(), self.encode_group_id(id)?)]
                            .into_iter()
                            .collect();
                    p.link(template_id, policy_id, env).context(
                        r#"Failed to link the policy template @id("permit-update-group")"#,
                    )?;
                    p
                };
                (action, resource, entities, policies)
            }
            UpdateGroupMembers { id, members } => {
                let action = engine.action_update_members.clone();
                let resource = self.encode_group_id(id)?;
                let entities = {
                    let principal = self.encode_principal_entity(by, std::iter::empty())?;
                    let update_group = self.encode_group_entity(id, members)?;
                    cedar_policy::Entities::from_entities([principal, update_group], None)
                        .context("Failed to make cedar entities")?
                };
                // TODO: let policies = { ... };
                (action, resource, entities, engine.policies.clone())
            }
        };
        let context = cedar_policy::Context::empty();
        let request = self.make_request(by, action, resource, context)?;
        let response = self
            .authorizer()
            .is_authorized(&request, &policies, &entities);
        Ok(self.read_response(response))
    }
}

// MARK: GroupAccessControl for Engine

impl<C, E> service::GroupAccessControl<C, E> for crate::Engine
where
    C: ProvideGroupEntityRepository<Error = E>,
    E: crate::Error,
{
    #[tracing::instrument(skip(self, ctx), ret(level = "debug"))]
    async fn judge_get_group(
        &self,
        ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
    ) -> Result<service::Judgement, E> {
        let r = Request::GetGroup(group_id);
        self.process_group_request(by, ctx, r).await
    }

    #[tracing::instrument(skip(self, ctx), ret(level = "debug"))]
    async fn judge_list_groups(
        &self,
        ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        let r = Request::ListGroups;
        self.process_group_request(by, ctx, r).await
    }

    #[tracing::instrument(skip(self, ctx, params), ret(level = "debug"))]
    async fn judge_create_group(
        &self,
        ctx: C,
        by: service::Principal,
        params: &domain::CreateGroupParams,
    ) -> Result<service::Judgement, E> {
        let r = Request::CreateGroup {
            members: &params.members,
        };
        self.process_group_request(by, ctx, r).await
    }

    #[tracing::instrument(skip(self, ctx, _params), ret(level = "debug"))]
    async fn judge_update_group(
        &self,
        ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
        _params: &domain::UpdateGroupParams,
    ) -> Result<service::Judgement, E> {
        let r = Request::UpdateGroup(group_id);
        self.process_group_request(by, ctx, r).await
    }

    #[tracing::instrument(skip(self, ctx, members), ret(level = "debug"))]
    async fn judge_update_group_members(
        &self,
        ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> Result<service::Judgement, E> {
        let r = Request::UpdateGroupMembers {
            id: group_id,
            members,
        };
        self.process_group_request(by, ctx, r).await
    }
}
