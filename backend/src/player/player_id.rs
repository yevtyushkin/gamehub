use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
/// Represents a player's ID.
pub struct PlayerId(pub Ulid);

impl PlayerId {
    /// Creates a random [PlayerId].
    pub fn new() -> PlayerId {
        PlayerId(Ulid::new())
    }
}
