use anyhow::Context;

use repository::Repository;

// MARK: structs

#[derive(Debug, Clone)]
pub struct State {
    service: service::Service,
    repository: Repository,
    authz: authz::Engine,
    pg_pool: sqlx::PgPool,
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceContext<'a> {
    repository: &'a Repository,
    authz: &'a authz::Engine,
    pg_pool: &'a sqlx::PgPool,
}

#[derive(Debug, Clone)]
pub struct AuthnState {
    service: service::AuthenticatedService,
    repository: Repository,
    authz: authz::Engine,
    pg_pool: sqlx::PgPool,
}

// MARK: impl State

impl State {
    pub async fn load_pg(config: &crate::config::PgConfig) -> anyhow::Result<Self> {
        let conn_opts = config.conn_options();
        let pool = sqlx::PgPool::connect_with(conn_opts)
            .await
            .context("Failed to connect to PostgreSQL")?;
        let repository = repository::Repository::up(&pool).await?;
        let authz = authz::Engine::new()?;
        Ok(Self {
            service: service::Service::new(),
            repository,
            authz,
            pg_pool: pool,
        })
    }

    fn service_context(&self) -> ServiceContext<'_> {
        ServiceContext {
            repository: &self.repository,
            authz: &self.authz,
            pg_pool: &self.pg_pool,
        }
    }
}

impl domain::ProvideUserService for State {
    type Context<'a>
        = ServiceContext<'a>
    where
        Self: 'a;
    type Error = crate::error::Error;
    type UserService<'a>
        = service::Service
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        self.service_context()
    }

    fn user_service(&self) -> &Self::UserService<'_> {
        &self.service
    }
}

impl service::MakeAuthenticated<crate::error::Error> for State {
    type Authenticated = AuthnState;

    async fn make_authenticated(
        &self,
        user_id: domain::UserId,
    ) -> Result<Self::Authenticated, crate::error::Error> {
        use service::ProvideUserRepository;

        let user = self.service_context().get_user(user_id).await?;
        Ok(AuthnState {
            service: self.service.authenticated(user.id),
            repository: self.repository.clone(),
            authz: self.authz.clone(),
            pg_pool: self.pg_pool.clone(),
        })
    }
}

// MARK: impl AuthnState

impl AuthnState {
    fn service_context(&self) -> ServiceContext<'_> {
        ServiceContext {
            repository: &self.repository,
            authz: &self.authz,
            pg_pool: &self.pg_pool,
        }
    }
}

impl domain::ProvideGroupService for AuthnState {
    type Context<'a>
        = ServiceContext<'a>
    where
        Self: 'a;
    type Error = crate::error::Error;
    type GroupService<'a>
        = service::AuthenticatedService
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        self.service_context()
    }

    fn group_service(&self) -> &Self::GroupService<'_> {
        &self.service
    }
}

impl domain::ProvideUserService for AuthnState {
    type Context<'a>
        = ServiceContext<'a>
    where
        Self: 'a;
    type Error = crate::error::Error;
    type UserService<'a>
        = service::AuthenticatedService
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        self.service_context()
    }

    fn user_service(&self) -> &Self::UserService<'_> {
        &self.service
    }
}

// MARK: impl ServiceContext

impl service::ProvideUserRepository for ServiceContext<'_> {
    type Context<'a>
        = &'a sqlx::PgPool
    where
        Self: 'a;
    type Error = crate::error::Error;
    type UserRepository<'a>
        = Repository
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        self.pg_pool
    }

    fn user_repository(&self) -> &Self::UserRepository<'_> {
        self.repository
    }
}

impl service::ProvideGroupRepository for ServiceContext<'_> {
    type Context<'a>
        = &'a sqlx::PgPool
    where
        Self: 'a;
    type Error = crate::error::Error;
    type GroupRepository<'a>
        = Repository
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        self.pg_pool
    }

    fn group_repository(&self) -> &Self::GroupRepository<'_> {
        self.repository
    }
}

impl service::ProvideUserAccessControl for ServiceContext<'_> {
    type Context<'a>
        = ()
    where
        Self: 'a;
    type Error = crate::error::Error;
    type UserAccessControl<'a>
        = authz::Engine
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {}

    fn user_access_control(&self) -> &Self::UserAccessControl<'_> {
        self.authz
    }
}

impl service::ProvideGroupAccessControl for ServiceContext<'_> {
    type Context<'a>
        = ()
    where
        Self: 'a;
    type Error = crate::error::Error;
    type GroupAccessControl<'a>
        = authz::Engine
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {}

    fn group_access_control(&self) -> &Self::GroupAccessControl<'_> {
        self.authz
    }
}
