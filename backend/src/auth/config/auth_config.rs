use id_token_verifier::prelude::IdTokenVerifierConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq)]
/// Represents an authentication module configuration.
pub struct AuthConfig {
    /// The Google [IdTokenVerifierConfig].
    pub google_id_token_verifier: IdTokenVerifierConfig,
}
