use crate::app_state::AppState;
use crate::players::error::PlayersError;
use crate::players::jwt_service::AuthTokenClaims;
use crate::players::players_service::PlayersService;
use axum::RequestPartsExt;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{DateTime, Utc};
use petname::{Generator, Petnames};
use serde::*;
use std::str::FromStr;
use uuid::Uuid;

/// Player representation in the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    /// [Player]'s ID.
    pub id: PlayerId,

    /// [Player]'s screen name.
    pub screen_name: PlayerScreenName,

    /// When a [Player] has joined.
    pub joined_at: PlayerJoinedAt,
}

impl Player {
    #[cfg(test)]
    /// Returns a test [Player].
    pub fn test() -> Player {
        Player {
            id: PlayerId::test(),
            screen_name: PlayerScreenName::test(),
            joined_at: PlayerJoinedAt::test(),
        }
    }
}

impl<S: AppState> FromRequestParts<S> for Player {
    type Rejection = PlayersError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let claims: AuthTokenClaims = parts.extract_with_state(state).await?;
        let player = state.players_service().player_by_id(&claims.sub).await?;
        Ok(player)
    }
}

/// [Player]'s ID.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, sqlx::Type)]
pub struct PlayerId(pub Uuid);

impl PlayerId {
    /// Creates a new random [PlayerId].
    pub fn random() -> PlayerId {
        PlayerId(Uuid::now_v7())
    }

    #[cfg(test)]
    /// Returns a test [PlayerId].
    pub fn test() -> PlayerId {
        PlayerId(Uuid::from_u128(1234567890))
    }
}

/// [Player]'s screen name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
pub struct PlayerScreenName(String);

impl PlayerScreenName {
    /// Creates a new random [PlayerScreenName].
    pub fn random() -> PlayerScreenName {
        let value = Petnames::small()
            .generate_one(3, "-")
            .unwrap_or_else(|| "no-screen-name".into());

        PlayerScreenName(value)
    }

    #[cfg(test)]
    /// Returns a test [PlayerScreenName].
    pub fn test() -> PlayerScreenName {
        PlayerScreenName("test-screen-name".into())
    }

    /// The maximum size of a [PlayerScreenName] in bytes.
    pub const MAX_SIZE: usize = 30;
}

/// [Player]'s screen name validation error.
#[derive(Debug, Clone, Copy, Eq, PartialEq, thiserror::Error)]
pub enum InvalidPlayerScreenName {
    /// [PlayerScreenName] is empty.
    #[error("player screen name is empty")]
    Empty,

    /// [PlayerScreenName] exceeds [PlayerScreenName::MAX_SIZE].
    #[error(
        "player screen name is larger than allowed max size of {} bytes",
        PlayerScreenName::MAX_SIZE
    )]
    ExceedsMaxSize,
}

impl FromStr for PlayerScreenName {
    type Err = InvalidPlayerScreenName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            Err(InvalidPlayerScreenName::Empty)?
        }
        if s.len() > PlayerScreenName::MAX_SIZE {
            Err(InvalidPlayerScreenName::ExceedsMaxSize)?
        }

        Ok(PlayerScreenName(s.into()))
    }
}

/// When the [Player] joined.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
pub struct PlayerJoinedAt(pub DateTime<Utc>);

impl PlayerJoinedAt {
    /// Creates a new [PlayerJoinedAt] with the current time.
    pub fn now() -> PlayerJoinedAt {
        PlayerJoinedAt(Utc::now())
    }

    #[cfg(test)]
    /// Returns a test [PlayerJoinedAt].
    pub fn test() -> PlayerJoinedAt {
        PlayerJoinedAt(DateTime::UNIX_EPOCH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_json_snapshot() {
        insta::assert_json_snapshot!(&Player::test());
    }

    #[test]
    fn player_screen_name_from_str_succeeds_if_screen_name_is_valid_of_min_size() {
        assert_eq!(
            PlayerScreenName::from_str("a"),
            Ok(PlayerScreenName("a".into()))
        );

        assert_eq!(
            PlayerScreenName::from_str(" a  "),
            Ok(PlayerScreenName("a".into()))
        );

        assert_eq!(
            PlayerScreenName::from_str("г"),
            Ok(PlayerScreenName("г".into()))
        );

        assert_eq!(
            PlayerScreenName::from_str("  г     "),
            Ok(PlayerScreenName("г".into()))
        );
    }

    #[test]
    fn player_screen_name_from_str_succeeds_if_screen_name_is_valid_of_max_size() {
        let value = "w".repeat(PlayerScreenName::MAX_SIZE);
        assert_eq!(
            PlayerScreenName::from_str(&value),
            Ok(PlayerScreenName(value.clone()))
        );

        let value_ws = format!("  {value}   ");
        assert_eq!(
            PlayerScreenName::from_str(&value_ws),
            Ok(PlayerScreenName(value))
        );

        let value = "г".repeat(PlayerScreenName::MAX_SIZE / 2);
        assert_eq!(
            PlayerScreenName::from_str(&value),
            Ok(PlayerScreenName(value.clone()))
        );

        let value_ws = format!("  {value}   ");
        assert_eq!(
            PlayerScreenName::from_str(&value_ws),
            Ok(PlayerScreenName(value))
        );
    }

    #[test]
    fn player_screen_name_from_str_succeeds_if_screen_name_is_valid_of_medium_size() {
        assert_eq!(
            PlayerScreenName::from_str("abcd1234"),
            Ok(PlayerScreenName("abcd1234".into()))
        );

        assert_eq!(
            PlayerScreenName::from_str("  abcd1234     "),
            Ok(PlayerScreenName("abcd1234".into()))
        );

        assert_eq!(
            PlayerScreenName::from_str("абв123"),
            Ok(PlayerScreenName("абв123".into()))
        );

        assert_eq!(
            PlayerScreenName::from_str("  абв123     "),
            Ok(PlayerScreenName("абв123".into()))
        );
    }

    #[test]
    fn player_screen_name_from_str_fails_if_screen_name_is_empty() {
        assert_eq!(
            PlayerScreenName::from_str(""),
            Err(InvalidPlayerScreenName::Empty)
        );

        assert_eq!(
            PlayerScreenName::from_str("     "),
            Err(InvalidPlayerScreenName::Empty)
        );
    }

    #[test]
    fn player_screen_name_from_str_fails_if_screen_name_exceeds_max_size() {
        let value = "w".repeat(PlayerScreenName::MAX_SIZE + 1);
        assert_eq!(
            PlayerScreenName::from_str(&value),
            Err(InvalidPlayerScreenName::ExceedsMaxSize)
        );

        let value_ws = format!("  {value}   ");
        assert_eq!(
            PlayerScreenName::from_str(&value_ws),
            Err(InvalidPlayerScreenName::ExceedsMaxSize)
        );

        let value = "г".repeat(PlayerScreenName::MAX_SIZE / 2 + 1);
        assert_eq!(
            PlayerScreenName::from_str(&value),
            Err(InvalidPlayerScreenName::ExceedsMaxSize)
        );

        let value_ws = format!("  {value}   ");
        assert_eq!(
            PlayerScreenName::from_str(&value_ws),
            Err(InvalidPlayerScreenName::ExceedsMaxSize)
        );
    }
}
