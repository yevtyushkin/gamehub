use crate::player::player_id::PlayerId;
use crate::player::player_name::PlayerName;

/// Represents a player.
pub struct Player {
    /// The [PlayerId] of this [Player].
    id: PlayerId,

    /// The [PlayerName] of this [Player].
    name: PlayerName,
}
