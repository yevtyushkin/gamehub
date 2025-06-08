use figment::providers::Env;
use serde::*;

/// Application configuration.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    /// Server configuration.
    pub server: ServerConfig,
    /// Postgres configuration.
    pub postgres: PostgresConfig,
}

impl Config {
    /// Loads [Config] from environment variables.
    #[allow(clippy::result_large_err)]
    pub fn from_env() -> figment::Result<Config> {
        figment::Figment::new()
            .merge(Env::raw().split("__"))
            .extract()
    }
}

/// Server configuration.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ServerConfig {
    /// Server host address.
    pub host: String,
    /// Server port.
    pub port: u16,
}

impl ServerConfig {
    /// Returns server listening address as `host:port`.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Postgres configuration.
#[derive(derive_more::Debug, Deserialize, Clone, PartialEq)]
pub struct PostgresConfig {
    /// Postgres host.
    pub host: String,
    /// Postgres port.
    pub port: u16,
    /// Postgres username.
    #[debug("<postgres_username_redacted>")]
    pub username: String,
    /// Postgres password.
    #[debug("<postgres_password_redacted>")]
    pub password: String,
    /// Postgres database name.
    pub database: String,
}

impl PostgresConfig {
    /// Returns Postgres connection URL as a string.
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env() {
        figment::Jail::expect_with(|j| {
            j.set_env("SERVER__HOST", "127.0.0.1");
            j.set_env("SERVER__PORT", "8080");

            j.set_env("POSTGRES__HOST", "127.0.0.1");
            j.set_env("POSTGRES__PORT", "5432");
            j.set_env("POSTGRES__USERNAME", "postgres");
            j.set_env("POSTGRES__PASSWORD", "postgres_password");
            j.set_env("POSTGRES__DATABASE", "postgres_database");

            let config = Config::from_env()?;
            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        host: "127.0.0.1".to_string(),
                        port: 8080
                    },
                    postgres: PostgresConfig {
                        host: "127.0.0.1".to_string(),
                        port: 5432,
                        username: "postgres".to_string(),
                        password: "postgres_password".to_string(),
                        database: "postgres_database".to_string(),
                    }
                }
            );

            Ok(())
        });
    }

    #[test]
    fn server_config_listen_addr() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
        };

        assert_eq!(config.listen_addr(), "127.0.0.1:8080");
    }

    #[test]
    fn postgres_config_connection_url() {
        let config = PostgresConfig {
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres_username".to_string(),
            password: "postgres_password".to_string(),
            database: "postgres_database".to_string(),
        };

        assert_eq!(
            config.connection_url(),
            "postgres://postgres_username:postgres_password@127.0.0.1:5432/postgres_database"
        );
    }

    #[test]
    fn config_debug_does_not_leak_sensitive_info() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            postgres: PostgresConfig {
                host: "127.0.0.1".to_string(),
                port: 5432,
                username: "postgres_1q2w3e4r_username".to_string(),
                password: "postgres_1q2w3e4r_password".to_string(),
                database: "postgres_database".to_string(),
            },
        };

        let debug = format!("{config:?}");

        assert!(!debug.contains("postgres_1q2w3e4r_username"));
        assert!(debug.contains("<postgres_username_redacted>"));

        assert!(!debug.contains("postgres_1q2w3e4r_password"));
        assert!(debug.contains("<postgres_password_redacted>"));
    }
}
