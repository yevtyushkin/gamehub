use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

mod config;
mod healthcheck;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env()?;
    info!("Starting app with config: {config:?}");

    let pg_pool = sqlx::PgPool::connect(&config.postgres.connection_url()).await?;
    sqlx::migrate!().run(&pg_pool).await?;

    let tcp_listener = TcpListener::bind(&config.server.listen_addr()).await?;
    let router = Router::new().merge(healthcheck::router());
    axum::serve(tcp_listener, router).await?;

    Ok(())
}
