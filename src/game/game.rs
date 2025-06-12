/// [Game] representation within the application.
#[derive(Debug, Clone, PartialEq)]
pub struct Game<S> {
    /// Common [Game] information: id, last activity, version, etc.
    pub common: Common,

    /// Game-specific game state, i.e., cards in card games, map in tic-tac-toe, etc.
    pub state: S,
}

/// Common [Game] data.
#[derive(Debug, Clone, PartialEq)]
pub struct Common {
    /// [Game]'s ID.
    id: GameId,

    /// When [Game] had the last activity.
    last_activity_at: LastActivityAt,
}

/// [Game]'s ID.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct GameId(pub uuid::Uuid);

impl GameId {
    /// Creates a new random [GameId].
    pub fn random() -> GameId {
        GameId(uuid::Uuid::now_v7())
    }

    #[cfg(test)]
    /// Returns a test [GameId].
    fn test() -> GameId {
        GameId(uuid::Uuid::from_u128(1234567890))
    }
}

/// When the [Game] had the last activity.
#[derive(Debug, Clone, PartialEq)]
pub struct LastActivityAt(pub chrono::DateTime<chrono::Utc>);

impl LastActivityAt {
    /// Creates a new [LastActivityAt] with the current time.
    pub fn now() -> LastActivityAt {
        LastActivityAt(chrono::Utc::now())
    }

    #[cfg(test)]
    /// Returns a test [LastActivityAt].
    fn test() -> LastActivityAt {
        LastActivityAt(chrono::DateTime::UNIX_EPOCH)
    }
}
