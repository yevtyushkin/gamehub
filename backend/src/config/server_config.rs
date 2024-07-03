use serde::Deserialize;

#[derive(Deserialize, Eq, PartialEq, Debug)]
/// Represents a server configuration.
pub struct ServerConfig {
    /// The host the server binds to.
    pub host: String,

    /// The port the server binds to.
    pub port: u16,
}
