mod group;
mod user;

#[derive(Debug, Clone)]
pub struct Repository {
    _priv: (),
}

impl Repository {
    const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    pub async fn up(pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        use anyhow::Context;

        Self::MIGRATOR
            .run(pool)
            .await
            .context("Failed to run migrations")?;
        Ok(Self { _priv: () })
    }

    async fn within_tx<F, T, E>(&self, pool: &sqlx::PgPool, f: F) -> Result<T, E>
    where
        F: AsyncFnOnce(&mut sqlx::PgConnection) -> Result<T, E>,
        E: crate::Error,
    {
        use anyhow::Context;

        let mut tx = pool
            .begin()
            .await
            .context("Failed to begin postgres transaction")?;
        let result = f(&mut tx).await;
        match result {
            Ok(v) => {
                tx.commit().await.context("Failed to commit transaction")?;
                Ok(v)
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .context("Failed to rollback transaction")?;
                Err(e)
            }
        }
    }
}

pub trait Error: domain::Error + From<anyhow::Error> {
    fn not_found(message: &str) -> Self;
}

pub trait AsPgPool: Send + Sync {
    fn as_pg_pool(&self) -> &sqlx::PgPool;
}

impl AsPgPool for sqlx::PgPool {
    fn as_pg_pool(&self) -> &sqlx::PgPool {
        self
    }
}

impl<P> AsPgPool for &P
where
    P: AsPgPool + ?Sized,
{
    fn as_pg_pool(&self) -> &sqlx::PgPool {
        P::as_pg_pool(self)
    }
}
