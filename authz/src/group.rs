use anyhow::Context;
use cedar_policy::EntityUid;

// MARK: GroupEngine

#[derive(Debug, Clone)]
pub(crate) struct GroupEngine {
    policies: cedar_policy::PolicySet,
    action_get: EntityUid,
    action_list: EntityUid,
    action_update: EntityUid,
    action_update_members: EntityUid,
    resource_create_group: EntityUid,
    resource_list_groups: EntityUid,
}

impl GroupEngine {
    pub(crate) const POLICIES: &str = include_str!("policies/group.cedar");
    pub(crate) const GET_ID: &str = "get-group";
    pub(crate) const LIST_ID: &str = "list-groups";
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

// MARK: GroupAccessControl for Engine

impl<C, E> service::GroupAccessControl<C, E> for crate::Engine
where
    C: ProvideGroupEntityRepository,
    E: crate::Error,
{
    async fn judge_get_group(
        &self,
        _ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }

    async fn judge_list_groups(
        &self,
        _ctx: C,
        by: service::Principal,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }

    async fn judge_create_group(
        &self,
        _ctx: C,
        by: service::Principal,
        params: &domain::CreateGroupParams,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }

    async fn judge_update_group(
        &self,
        _ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
        params: &domain::UpdateGroupParams,
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }

    async fn judge_update_group_members(
        &self,
        _ctx: C,
        by: service::Principal,
        group_id: domain::GroupId,
        members: &[domain::UserId],
    ) -> Result<service::Judgement, E> {
        Err(anyhow::anyhow!("not implemented").into())
    }
}
