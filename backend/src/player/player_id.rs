use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
/// Represents a player's ID.
pub struct PlayerId(pub Uuid);

impl PlayerId {
    /// Creates a random [PlayerId].
    pub fn new() -> PlayerId {
        PlayerId(Uuid::now_v7())
    }
}
