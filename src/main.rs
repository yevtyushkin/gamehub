use crate::app_state::AppStateDefault;
use crate::players::jwt_service::JwtServiceDefault;
use crate::players::players_service::PlayersServiceDefault;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

mod api_error;
mod app_state;
mod config;
mod healthcheck;
mod players;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env()?;
    info!("Starting app with config: {config:?}");

    let pg_pool = sqlx::PgPool::connect(&config.postgres.connection_url()).await?;
    sqlx::migrate!().run(&pg_pool).await?;

    let http_client = reqwest::Client::new();
    let jwt_service = JwtServiceDefault::new(config.jwt);
    let players_service = PlayersServiceDefault::new(
        pg_pool,
        http_client,
        config.google_id_token_verifier,
        jwt_service.clone(),
    );
    let app_state = Arc::new(AppStateDefault::new(players_service, jwt_service));

    let tcp_listener = TcpListener::bind(&config.server.listen_addr()).await?;
    let router = Router::new().merge(healthcheck::router()).merge(
        Router::new()
            .merge(players::http::router())
            .with_state(app_state),
    );
    axum::serve(tcp_listener, router).await?;

    Ok(())
}
