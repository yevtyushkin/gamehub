use crate::app_state::AppState;
use crate::players::error::PlayersError;
use crate::players::jwt_service::AuthToken;
use crate::players::player::Player;
use crate::players::players_service::PlayersService;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::*;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// A sign-in request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignInRequest {
    /// Sign in with Google.
    Google {
        /// Google issued [IdToken].
        id_token: IdToken,
    },
}

impl SignInRequest {
    #[cfg(test)]
    /// Returns a test [SignInRequest::Google].
    pub fn test_google() -> SignInRequest {
        SignInRequest::Google {
            id_token: IdToken::test(),
        }
    }
}

/// Response to a [SignInRequest] in case of success.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignInResponse<'a> {
    /// [AuthToken] to use in subsequent requests.
    pub auth_token: AuthToken<'a>,
}

impl SignInResponse<'_> {
    #[cfg(test)]
    /// Returns a test [SignInResponse].
    pub fn test() -> SignInResponse<'static> {
        SignInResponse {
            auth_token: AuthToken::test(),
        }
    }
}

/// ID token containing user information.
#[derive(
    Debug, Clone, Deserialize, Serialize, derive_more::AsRef, derive_more::Deref, PartialEq,
)]
pub struct IdToken(pub String);

impl IdToken {
    #[cfg(test)]
    /// Returns a test [IdToken].
    pub fn test() -> IdToken {
        IdToken("test-id-token".into())
    }
}

/// [Router] for the [crate::players] module.
pub fn router<S: AppState>() -> Router<S> {
    Router::new().nest(
        "/players",
        Router::new()
            .route("/sign_in", post(sign_in::<S>))
            .route("/player_info", get(player_info)),
    )
}

/// `/sign_in` handler. Handles [SignInRequest] and returns [SignInResponse] in case of success.
async fn sign_in<S: AppState>(
    State(app_state): State<S>,
    Json(request): Json<SignInRequest>,
) -> Result<Response, PlayersError> {
    let auth_token = app_state.players_service().sign_in(&request).await?;

    let body = SignInResponse { auth_token };
    let response = (StatusCode::OK, Json(body)).into_response();

    Ok(response)
}

/// `/player_info` handler. Returns current [Player] information.
async fn player_info(player: Player) -> Json<Player> {
    Json(player)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_error::ApiError;
    use crate::app_state::MockAppState;
    use crate::players::jwt_service::{AuthTokenClaims, MockJwtService};
    use crate::players::player::PlayerId;
    use crate::players::players_service::MockPlayersService;
    use axum_test::TestServer;
    use axum_test::http::header::AUTHORIZATION;
    use id_token_verifier::IdTokenVerifierError;
    use id_token_verifier::validation::ValidationError;
    use jsonwebtoken::errors::ErrorKind;
    use mockall::predicate::eq;
    use std::sync::Arc;

    #[test]
    fn sign_in_request_json_snapshot() {
        insta::assert_json_snapshot!(&SignInRequest::test_google());
    }

    #[test]
    fn sign_in_response_json_snapshot() {
        insta::assert_json_snapshot!(&SignInResponse::test());
    }

    #[tokio::test]
    async fn sign_in_handler_returns_correct_response_when_players_service_succeds()
    -> anyhow::Result<()> {
        let mut players_service = MockPlayersService::new();
        players_service
            .expect_sign_in()
            .with(eq(SignInRequest::test_google()))
            .returning(|_| Box::pin(async { Ok(AuthToken::test()) }));
        let state = Arc::new(MockAppState::default().with_players_service(players_service));
        let server = TestServer::new(router().with_state(state))?;

        let response = server
            .post("/players/sign_in")
            .json(&SignInRequest::test_google())
            .await;

        response.assert_status(StatusCode::OK);
        response.assert_json(&SignInResponse::test());

        Ok(())
    }

    #[tokio::test]
    async fn sign_in_handler_returns_correct_response_when_players_service_fails()
    -> anyhow::Result<()> {
        let mut players_service = MockPlayersService::new();
        players_service
            .expect_sign_in()
            .with(eq(SignInRequest::test_google()))
            .returning(|_| {
                Box::pin(async {
                    Err(PlayersError::IdToken(IdTokenVerifierError::Validation(
                        ValidationError::MissingKeyId,
                    )))
                })
            });
        let state = Arc::new(MockAppState::default().with_players_service(players_service));
        let server = TestServer::new(router().with_state(state))?;

        let response = server
            .post("/players/sign_in")
            .json(&SignInRequest::test_google())
            .await;

        response.assert_status(StatusCode::BAD_REQUEST);
        let error = response.json::<ApiError>();
        assert_eq!(error.module, "players");
        assert_eq!(error.id, 0);

        Ok(())
    }

    #[tokio::test]
    async fn player_info_handler_fails_if_auth_token_missing() -> anyhow::Result<()> {
        let state = Arc::new(MockAppState::default());
        let server = TestServer::new(router().with_state(state))?;

        let response = server.get("/players/player_info").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
        let error = response.json::<ApiError>();
        assert_eq!(error.module, "players");
        assert_eq!(error.id, 3);
        assert_eq!(error.dev_message, "auth token is missing");

        Ok(())
    }

    #[tokio::test]
    async fn player_info_handler_fails_if_auth_token_validation_fails() -> anyhow::Result<()> {
        let mut jwt_service = MockJwtService::new();
        jwt_service
            .expect_verify_token()
            .withf(|token| token.as_ref() == "invalid")
            .returning(|_| {
                Err(PlayersError::AuthToken(jsonwebtoken::errors::Error::from(
                    ErrorKind::InvalidToken,
                )))
            });

        let state = Arc::new(MockAppState::default().with_jwt_service(jwt_service));
        let mut server = TestServer::new(router().with_state(state))?;
        server.add_header(AUTHORIZATION, "Bearer invalid");

        let response = server.get("/players/player_info").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
        let error = response.json::<ApiError>();
        assert_eq!(error.module, "players");
        assert_eq!(error.id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn player_info_handler_fails_if_player_lookup_fails() -> anyhow::Result<()> {
        let mut jwt_service = MockJwtService::new();
        jwt_service
            .expect_verify_token()
            .withf(|token| token.as_ref() == "valid")
            .returning(|_| Ok(AuthTokenClaims::test()));

        let mut players_service = MockPlayersService::new();
        players_service
            .expect_player_by_id()
            .withf(|player_id| player_id == &PlayerId::test())
            .returning(|_| Box::pin(async { Err(PlayersError::PlayerNotFound) }));

        let state = Arc::new(
            MockAppState::default()
                .with_jwt_service(jwt_service)
                .with_players_service(players_service),
        );
        let mut server = TestServer::new(router().with_state(state))?;
        server.add_header(AUTHORIZATION, "Bearer valid");

        let response = server.get("/players/player_info").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
        let error = response.json::<ApiError>();
        assert_eq!(error.module, "players");
        assert_eq!(error.id, 1);

        Ok(())
    }
}
