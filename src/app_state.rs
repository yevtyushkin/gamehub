use crate::players::jwt_service::{JwtService, JwtServiceDefault};
use crate::players::players_service::{PlayersService, PlayersServiceDefault};
use std::sync::Arc;

/// Application state, shared between HTTP handlers.
pub trait AppState: Clone + Send + Sync + 'static {
    /// [PlayersService] implementation.
    type PlayersService: PlayersService + Send + Sync + 'static;
    /// Returns a reference to [Self::PlayersService] implementation.
    fn players_service(&self) -> &Self::PlayersService;

    /// [JwtService] implementation.
    type JwtService: JwtService + Send + Sync + 'static;
    /// Returns a reference to [Self::JwtService] implementation.
    fn jwt_service(&self) -> &Self::JwtService;
}

/// Default [AppState] implementation.
#[derive(Default)]
#[cfg_attr(test, derive(getset::WithSetters))]
#[cfg_attr(test, getset(set_with = "pub"))]
pub struct AppStateDefault<PS = PlayersServiceDefault, JS = JwtServiceDefault> {
    /// [PlayersService] implementation.
    players_service: PS,

    /// [JwtService] implementation.
    jwt_service: JS,
}

#[cfg(test)]
pub type MockAppState = AppStateDefault<
    crate::players::players_service::MockPlayersService,
    crate::players::jwt_service::MockJwtService,
>;

impl<PS, JS> AppStateDefault<PS, JS> {
    /// Creates a new [AppStateDefault] with the given services.
    pub fn new(players_service: PS, jwt_service: JS) -> AppStateDefault<PS, JS> {
        AppStateDefault {
            players_service,
            jwt_service,
        }
    }
}

impl<PS, JS> AppState for Arc<AppStateDefault<PS, JS>>
where
    PS: PlayersService + Send + Sync + 'static,
    JS: JwtService + Send + Sync + 'static,
{
    type PlayersService = PS;
    fn players_service(&self) -> &Self::PlayersService {
        &self.players_service
    }

    type JwtService = JS;
    fn jwt_service(&self) -> &Self::JwtService {
        &self.jwt_service
    }
}
