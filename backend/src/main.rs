use axum::Router;
use tokio::net::TcpListener;

use crate::config::application_config::ApplicationConfig;
use crate::health_check::health_check_routes::health_check_routes;

mod auth;
mod config;
mod health_check;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let application_config = ApplicationConfig::from_environment()?;

    let tcp_listener = {
        let server_conf = &application_config.common.server;
        let addr = format!("{}:{}", &server_conf.host, &server_conf.port);
        TcpListener::bind(addr).await?
    };
    axum::serve(tcp_listener, Router::new().merge(health_check_routes())).await?;

    Ok(())
}
