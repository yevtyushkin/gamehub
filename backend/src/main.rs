use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::{migrate, PgPool, Pool, Postgres};
use tokio::net::TcpListener;

use crate::config::application_config::ApplicationConfig;
use crate::config::postgres_config::PostgresConfig;
use crate::config::server_config::ServerConfig;
use crate::health_check::health_check_routes::health_check_routes;

mod auth;
mod config;
mod health_check;
mod player;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application_config = ApplicationConfig::from_environment()?;

    let pg_pool = create_pg_pool_and_migrate(application_config.common.postgres).await?;
    let tcp_listener = create_server_tcp_listener(&application_config.common.server).await?;

    axum::serve(tcp_listener, Router::new().merge(health_check_routes())).await?;

    Ok(())
}

/// Creates a [PgPool] using the given [postgres_config] and applies pending migrations from the ./migrations folder.
async fn create_pg_pool_and_migrate(
    postgres_config: PostgresConfig,
) -> Result<PgPool, sqlx::Error> {
    let postgres_connection_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        postgres_config.username,
        postgres_config.password.value,
        postgres_config.host,
        postgres_config.port,
        postgres_config.database
    );

    let pg_pool = PgPoolOptions::new()
        .max_connections(postgres_config.max_connections)
        .connect(&postgres_connection_url)
        .await?;

    migrate!("./migrations").run(&pg_pool).await?;

    Ok(pg_pool)
}

/// Creates a server [TcpListener] using the given [server_config].
async fn create_server_tcp_listener(server_config: &ServerConfig) -> std::io::Result<TcpListener> {
    let addr = format!("{}:{}", &server_config.host, &server_config.port);

    TcpListener::bind(&addr).await
}
