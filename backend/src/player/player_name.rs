use std::borrow::Cow;

use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::player::player_name_validation_error::PlayerNameValidationError;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Cow<'_, str>")]
/// Represents a player's name.
pub struct PlayerName(String);

impl PlayerName {
    /// The minimum length of a [PlayerName] in bytes.
    pub const MIN_LENGTH: usize = 2;

    /// The maximum length of a [PlayerName] in bytes.
    pub const MAX_LENGTH: usize = 20;

    /// Sanitizes, validates and returns a new instance of [PlayerName] from the given string or a [PlayerNameValidationError] otherwise.
    pub fn from_str(value: &str) -> Result<PlayerName, PlayerNameValidationError> {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            Err(PlayerNameValidationError::Empty)?
        } else if trimmed.len() < Self::MIN_LENGTH {
            Err(PlayerNameValidationError::TooShort)?
        } else if trimmed.len() > Self::MAX_LENGTH {
            Err(PlayerNameValidationError::TooLong)?
        } else {
            Ok(PlayerName(trimmed.into()))
        }
    }
}

impl<'a> TryFrom<Cow<'a, str>> for PlayerName {
    type Error = PlayerNameValidationError;

    fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
        PlayerName::from_str(&value)
    }
}

mod tests {
    use crate::player::player_name::PlayerName;
    use crate::player::player_name_validation_error::PlayerNameValidationError;
    use serde_json::{from_str, from_value, json};

    #[test]
    fn from_str_does_not_allow_empty_name() {
        let result = PlayerName::from_str("");

        assert_eq!(result, Err(PlayerNameValidationError::Empty));
    }

    #[test]
    fn from_str_does_not_allow_too_short_name() {
        let result = PlayerName::from_str("1");

        assert_eq!(result, Err(PlayerNameValidationError::TooShort));
    }

    #[test]
    fn from_str_sanitizes_and_allows_correct_name() {
        let result = PlayerName::from_str(" 12");
        assert_eq!(result, Ok(PlayerName("12".to_string())));

        let result = PlayerName::from_str("123 ");
        assert_eq!(result, Ok(PlayerName("123".to_string())));

        let result = PlayerName::from_str(" 01234567890123456789");
        assert_eq!(result, Ok(PlayerName("01234567890123456789".to_string())));
    }

    #[test]
    fn from_str_does_not_allow_too_long_name() {
        let result = PlayerName::from_str("01234567890123456789X");

        assert_eq!(result, Err(PlayerNameValidationError::TooLong));
    }

    #[test]
    fn player_name_can_be_deserialized() {
        let result = from_str::<'_, PlayerName>("\"12\"").unwrap();
        assert_eq!(result, PlayerName("12".to_string()));

        let result = from_value::<PlayerName>(json!("123")).unwrap();
        assert_eq!(result, PlayerName("123".to_string()));
    }
}
