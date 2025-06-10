use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Outbound API error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError<'a> {
    /// Module where the error has occurred, i.e. "players", "game", "matchmaking", etc.
    pub module: Cow<'a, str>,

    /// Unique error ID within the error module.
    pub id: u32,

    /// Error message for the developer.
    pub dev_message: Cow<'a, str>,
}
