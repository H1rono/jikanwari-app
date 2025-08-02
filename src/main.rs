mod config;
mod error;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let pg_config = config::PgConfig::load_env("POSTGRES_")?;
    let state = state::State::load_pg(&pg_config).await?;
    let router = router::Service::new(state).into_router();
    let serve_config = config::ServeConfig::load_env("")?;
    let addr = serve_config.socket_addr();
    tracing::info!("Starting server at {addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind TCP listener")?;
    axum::serve(listener, router)
        .with_graceful_shutdown(signal())
        .await
        .context("Failed to start server")?;
    Ok(())
}

#[tracing::instrument]
async fn signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => tracing::info!("Received Ctrl+C signal"),
        Err(e) => tracing::error!(error = %e, "Failed to listen for Ctrl+C signal"),
    }
}
