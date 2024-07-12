use thiserror::Error;

use crate::player::player_name::PlayerName;

/// Enumerates possible [PlayerName] validation errors.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PlayerNameValidationError {
    /// The error returned when the [PlayerName] is empty.
    #[error("Player name must not be empty")]
    Empty,

    /// The error returned when the [PlayerName] is too short.
    #[error(
        "Player name must not be less than {} in bytes",
        PlayerName::MIN_LENGTH
    )]
    TooShort,

    /// The error returned when the [PlayerName] is too long.
    #[error(
        "Player name must not be longer than {} in bytes",
        PlayerName::MAX_LENGTH
    )]
    TooLong,
}
