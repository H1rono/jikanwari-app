use anyhow::Context;

#[derive(Debug, Clone)]
pub struct State<R> {
    service: service::Service,
    repository: R,
}

impl State<repository::Repository> {
    pub async fn load_pg(config: &crate::config::PgConfig) -> anyhow::Result<Self> {
        let conn_opts = config.conn_options();
        let pool = sqlx::PgPool::connect_with(conn_opts)
            .await
            .context("Failed to connect to PostgreSQL")?;
        let repository = repository::Repository::up(pool).await?;
        Ok(Self {
            service: service::Service::new(),
            repository,
        })
    }
}

impl<R> domain::ProvideUserService for State<R>
where
    R: service::UserRepository<crate::error::Error>,
{
    type Context<'a>
        = &'a R
    where
        Self: 'a;
    type Error = crate::error::Error;
    type UserService<'a>
        = service::Service
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        &self.repository
    }

    fn user_service(&self) -> &Self::UserService<'_> {
        &self.service
    }
}

impl<R> domain::ProvideGroupService for State<R>
where
    R: service::GroupRepository<crate::error::Error>,
{
    type Context<'a>
        = &'a R
    where
        Self: 'a;
    type Error = crate::error::Error;
    type GroupService<'a>
        = service::Service
    where
        Self: 'a;

    fn context(&self) -> Self::Context<'_> {
        &self.repository
    }

    fn group_service(&self) -> &Self::GroupService<'_> {
        &self.service
    }
}
