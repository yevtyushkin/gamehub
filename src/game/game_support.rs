/// This trait must be implemented by each game to be supported by the application.
///
/// It defines associated state, WS messages, and other game-specific types, as well as how the
/// specific game handles commands and the lifecycle.
pub trait GameSupport {
    /// Incoming WS message of the game.
    type WsMsgIn;

    /// Commands processed by the game.
    type Command;

    /// Game specific state of the game.
    type GameState;
}
