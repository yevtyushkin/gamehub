use chrono::Duration;
use figment::providers::Env;
use id_token_verifier::IdTokenVerifierConfig;
use serde::*;

/// Application configuration.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    /// Server configuration.
    pub server: ServerConfig,

    /// Postgres configuration.
    pub postgres: PostgresConfig,

    /// JWT configuration.
    pub jwt: JwtConfig,

    /// Google ID token verifier configuration.
    pub google_id_token_verifier: IdTokenVerifierConfig,
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

/// JWT configuration.
#[derive(derive_more::Debug, Deserialize, Clone, PartialEq)]
pub struct JwtConfig {
    /// JWT secret for signing and verifying JWT tokens.
    #[debug("<jwt_secret_redacted>")]
    pub secret: String,

    /// TTL for JWT tokens.
    #[serde(deserialize_with = "duration_str::deserialize_duration_chrono")]
    pub ttl: Duration,
}

impl JwtConfig {
    #[cfg(test)]
    /// Returns a test [JwtConfig].
    pub fn test() -> JwtConfig {
        JwtConfig {
            secret: "jwt_secret".to_string(),
            ttl: Duration::seconds(3600),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use id_token_verifier::backoff_config::{BackoffConfig, ExponentialBackoffConfig};
    use id_token_verifier::cache::JwksCacheConfig;
    use id_token_verifier::client::{JwksClientConfig, JwksUrl};
    use id_token_verifier::validation::{Aud, Iss, ValidationConfig};

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

            j.set_env("JWT__SECRET", "jwt_secret");
            j.set_env("JWT__TTL", "1h");

            j.set_env(
                "GOOGLE_ID_TOKEN_VERIFIER__CLIENT__JWKS_URL__Discover",
                "https://accounts.google.com/.well-known/openid-configuration",
            );

            j.set_env(
                "GOOGLE_ID_TOKEN_VERIFIER__CLIENT__BACKOFF__STRATEGY",
                "Exponential",
            );

            j.set_env(
                "GOOGLE_ID_TOKEN_VERIFIER__VALIDATION__ALLOWED_ISS",
                "[\"https://accounts.google.com\", \"accounts.google.com\"]",
            );
            j.set_env(
                "GOOGLE_ID_TOKEN_VERIFIER__VALIDATION__ALLOWED_AUD",
                "gamehub_google_aud",
            );

            j.set_env("GOOGLE_ID_TOKEN_VERIFIER__CACHE__ENABLED", "true");
            j.set_env(
                "GOOGLE_ID_TOKEN_VERIFIER__VERIFIER_NAME",
                "google-id-token-verifier",
            );

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
                    },
                    jwt: JwtConfig {
                        secret: "jwt_secret".to_string(),
                        ttl: Duration::hours(1),
                    },
                    google_id_token_verifier: IdTokenVerifierConfig {
                        client: JwksClientConfig {
                            jwks_url: JwksUrl::Discover(
                                "https://accounts.google.com/.well-known/openid-configuration"
                                    .parse()
                                    .unwrap()
                            ),
                            backoff: BackoffConfig::Exponential(ExponentialBackoffConfig::default())
                        },
                        validation: ValidationConfig::builder()
                            .allowed_iss(vec![
                                Iss("https://accounts.google.com".to_string()),
                                Iss("accounts.google.com".to_string()),
                            ])
                            .allowed_aud(Aud("gamehub_google_aud".to_string()))
                            .build(),
                        cache: JwksCacheConfig::builder().build(),
                        verifier_name: Some("google-id-token-verifier".to_string()),
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
            jwt: JwtConfig {
                secret: "jwt_1q2w3e4r_secret".to_string(),
                ttl: Duration::hours(1),
            },
            google_id_token_verifier: IdTokenVerifierConfig {
                client: JwksClientConfig {
                    jwks_url: JwksUrl::Discover(
                        "https://accounts.google.com/.well-known/openid-configuration"
                            .parse()
                            .unwrap(),
                    ),
                    backoff: BackoffConfig::Exponential(ExponentialBackoffConfig::default()),
                },
                validation: ValidationConfig::builder()
                    .allowed_iss(vec![Iss("https://accounts.google.com".to_string())])
                    .allowed_aud(Aud("gamehub_google_aud".to_string()))
                    .build(),
                cache: JwksCacheConfig::builder().build(),
                verifier_name: Some("google-id-token-verifier".to_string()),
            },
        };

        let debug = format!("{config:?}");

        assert!(!debug.contains("postgres_1q2w3e4r_username"));
        assert!(debug.contains("<postgres_username_redacted>"));

        assert!(!debug.contains("postgres_1q2w3e4r_password"));
        assert!(debug.contains("<postgres_password_redacted>"));

        assert!(!debug.contains("jwt_1q2w3e4r_secret"));
        assert!(debug.contains("<jwt_secret_redacted>"));
    }
}
