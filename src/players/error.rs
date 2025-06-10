use crate::api_error::ApiError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Possible players errors.
#[derive(Debug, thiserror::Error)]
pub enum PlayersError {
    /// Player not found.
    #[error("player not found")]
    PlayerNotFound,

    /// Error when verifying third party id token.
    #[error("error when verifying third party id token: {0}")]
    IdToken(#[from] id_token_verifier::IdTokenVerifierError),

    /// Error when verifying auth token token.
    #[error("error when verifying auth token: {0}")]
    AuthToken(jsonwebtoken::errors::Error),

    /// When auth token is missing.
    #[error("auth token is missing")]
    AuthTokenMissing,

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for PlayersError {
    fn into_response(self) -> Response {
        let (status, id) = match &self {
            PlayersError::IdToken(_) => (StatusCode::BAD_REQUEST, 0),
            PlayersError::PlayerNotFound => (StatusCode::UNAUTHORIZED, 1),
            PlayersError::AuthToken(_) => (StatusCode::UNAUTHORIZED, 2),
            PlayersError::AuthTokenMissing => (StatusCode::UNAUTHORIZED, 3),
            PlayersError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 4),
        };

        let body = ApiError {
            module: "players".into(),
            id,
            dev_message: self.to_string().into(),
        };

        (status, Json(body)).into_response()
    }
}
