use crate::auth::config::auth_config::AuthConfig;
use crate::config::common_config::CommonConfig;
use config::{Config, ConfigError, Environment};
use serde::Deserialize;

/// The prefix used for this application's environment variables.
const CONFIG_PREFIX: &str = "GAMEHUB";

/// This separator used to represent nested configuration variables in environment variables.
///
/// # Example
///
/// Consider the following example of a nested configuration:
///
/// ```rust
/// struct Config {
///     nested: Nested
/// }
///
/// struct Nested {
///     var: String
/// }
/// ```
/// With this separator, the variable `Config::nested::var` would be represented as `NESTED__VAR` in environment variables.
const CONFIG_SEPARATOR: &str = "__";

#[derive(Deserialize, Eq, PartialEq, Debug)]
/// Represents a root application configuration.
pub struct ApplicationConfig {
    /// The common configuration of the application.
    pub common: CommonConfig,
    /// The auth module configuration of the application.
    pub auth: AuthConfig,
}

impl ApplicationConfig {
    /// Attempts to gather an [ApplicationConfig] from environment variables.
    pub fn from_environment() -> Result<ApplicationConfig, ConfigError> {
        let environment = Environment::with_prefix(CONFIG_PREFIX).separator(CONFIG_SEPARATOR);

        Config::builder()
            .add_source(environment)
            .build()?
            .try_deserialize::<'_, ApplicationConfig>()
    }
}

mod tests {
    use crate::auth::config::auth_config::AuthConfig;
    use crate::config::application_config::ApplicationConfig;
    use crate::config::common_config::CommonConfig;
    use crate::config::server_config::ServerConfig;
    use chrono::TimeDelta;
    use id_token_verifier::prelude::{IdTokenVerifierConfig, JwksUriType};
    use url::Url;

    #[test]
    fn from_environment_parses_application_config() {
        let server_host = String::from("test_host");
        std::env::set_var("GAMEHUB__COMMON__SERVER__HOST", &server_host);
        let server_port = 1234;
        std::env::set_var("GAMEHUB__COMMON__SERVER__PORT", server_port.to_string());

        let auth_google_id_token_verifier_jwks_uri_type = "AutoDiscover";
        std::env::set_var(
            "GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__JWKS_URI_TYPE",
            auth_google_id_token_verifier_jwks_uri_type.to_string(),
        );
        let auth_google_id_token_verifier_jwks_uri =
            "https://accounts.google.com/.well-known/openid-configuration";
        std::env::set_var(
            "GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__JWKS_URI",
            auth_google_id_token_verifier_jwks_uri.to_string(),
        );
        let auth_google_id_token_verifier_jwks_max_age = 1337;
        std::env::set_var(
            "GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__JWKS_MAX_AGE",
            auth_google_id_token_verifier_jwks_max_age.to_string(),
        );
        let auth_google_id_token_verifier_iss =
            vec!["goog_iss1".to_string(), "goog_iss2".to_string()];
        std::env::set_var(
            "GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__ISS",
            auth_google_id_token_verifier_iss.join(",").to_string(),
        );
        let auth_google_id_token_verifier_aud =
            vec!["goog_aud1".to_string(), "goog_aud2".to_string()];
        std::env::set_var(
            "GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__AUD",
            auth_google_id_token_verifier_aud.join(",").to_string(),
        );
        let auth_google_id_token_verifier_allow_unsafe_configuration = true;
        std::env::set_var(
            "GAMEHUB__AUTH__GOOGLE_ID_TOKEN_VERIFIER__ALLOW_UNSAFE_CONFIGURATION",
            auth_google_id_token_verifier_allow_unsafe_configuration.to_string(),
        );

        let expected_config = ApplicationConfig {
            common: CommonConfig {
                server: ServerConfig {
                    host: server_host,
                    port: server_port,
                },
            },
            auth: AuthConfig {
                google_id_token_verifier: IdTokenVerifierConfig {
                    jwks_uri_type: JwksUriType::AutoDiscover,
                    jwks_uri: Url::parse(auth_google_id_token_verifier_jwks_uri).unwrap(),
                    jwks_max_age: Some(TimeDelta::seconds(
                        auth_google_id_token_verifier_jwks_max_age,
                    )),
                    iss: auth_google_id_token_verifier_iss,
                    aud: auth_google_id_token_verifier_aud,
                    allow_unsafe_configuration:
                        auth_google_id_token_verifier_allow_unsafe_configuration,
                },
            },
        };

        let result = ApplicationConfig::from_environment();

        assert_eq!(result.unwrap(), expected_config);
    }
}
