use crate::config::secret_config::SecretConfig;
use serde::Deserialize;

/// Represents a Postgres config.
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct PostgresConfig {
    /// The host of the Postgres instance.
    pub host: String,

    /// The port of the Postgres instance.
    pub port: u16,

    /// The database name of the Postgres instance.
    pub database: String,

    /// The username of the Postgres instance.
    pub username: String,

    /// The password of the Postgres instance user.
    pub password: SecretConfig<String>,

    /// The maximum number of connections to allow to the Postgres instance.
    pub max_connections: u32,
}
