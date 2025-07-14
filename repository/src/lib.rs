mod user;

#[derive(Debug, Clone)]
pub struct Repository {
    pool: sqlx::PgPool,
}

impl Repository {
    const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    pub async fn up(pool: sqlx::PgPool) -> anyhow::Result<Self> {
        use anyhow::Context;

        Self::MIGRATOR
            .run(&pool)
            .await
            .context("Failed to run migrations")?;
        Ok(Self { pool })
    }
}

pub trait Error: domain::Error + From<anyhow::Error> {
    fn not_found(message: &str) -> Self;
}
