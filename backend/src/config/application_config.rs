use config::{Config, ConfigError, Environment};
use serde::Deserialize;

use crate::config::server_config::ServerConfig;

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
    /// The main server configuration of the application.
    pub server: ServerConfig,
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
    use crate::config::application_config::ApplicationConfig;
    use crate::config::server_config::ServerConfig;

    #[test]
    fn from_environment_parses_application_config() {
        let server_host = String::from("test_host");
        std::env::set_var("GAMEHUB__SERVER__HOST", &server_host);

        let server_port = 1234;
        std::env::set_var("GAMEHUB__SERVER__PORT", server_port.to_string());

        let expected_config = ApplicationConfig {
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
        };

        let result = ApplicationConfig::from_environment();

        assert_eq!(result.unwrap(), expected_config);
    }
}
