use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PgConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl PgConfig {
    pub fn load_env(prefix: &str) -> anyhow::Result<Self> {
        use anyhow::Context;

        let var = |key: &str| {
            std::env::var(format!("{prefix}{key}"))
                .with_context(|| format!("Environment variable {prefix}{key} not set"))
        };
        let host = var("HOSTNAME")?;
        let port = var("PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .context("Invalid port number")?;
        let user = var("USER")?;
        let password = var("PASSWORD")?;
        let database = var("DATABASE")?;

        Ok(Self {
            host,
            port,
            user,
            password,
            database,
        })
    }

    pub fn conn_options(&self) -> sqlx::postgres::PgConnectOptions {
        sqlx::postgres::PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .password(&self.password)
            .database(&self.database)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct ServeConfig {
    pub addr: std::net::IpAddr,
    pub port: u16,
}

impl ServeConfig {
    pub fn load_env(prefix: &str) -> anyhow::Result<Self> {
        use anyhow::Context;

        let var = |key: &str| {
            std::env::var(format!("{prefix}{key}"))
                .with_context(|| format!("Environment variable {prefix}{key} not set"))
        };
        let addr = var("ADDR")?.parse().context("Invalid IP address format")?;
        let port = var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("Invalid port number")?;

        Ok(Self { addr, port })
    }

    pub fn socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::new(self.addr, self.port)
    }
}
