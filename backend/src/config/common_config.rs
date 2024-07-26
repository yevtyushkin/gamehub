use crate::config::postgres_config::PostgresConfig;
use crate::config::server_config::ServerConfig;
use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Debug)]
/// Represents a common application configuration.
pub struct CommonConfig {
    /// The main server configuration of the application.
    pub server: ServerConfig,

    /// The Postgres configuration of the application.
    pub postgres: PostgresConfig,
}
