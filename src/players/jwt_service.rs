use crate::app_state::AppState;
use crate::config::JwtConfig;
use crate::players::error::PlayersError;
use crate::players::player::PlayerId;
use anyhow::Context;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Operations with [Player]s JWT tokens.
#[cfg_attr(test, mockall::automock)]
pub trait JwtService {
    /// Creates an [AuthToken] from the given [PlayerId].
    fn create_token(&self, player_id: PlayerId) -> Result<AuthToken<'static>, PlayersError>;

    /// Verifies the given [AuthToken] and returns its [AuthTokenClaims].
    #[allow(clippy::needless_lifetimes)]
    fn verify_token<'a>(&self, token: &AuthToken<'a>) -> Result<AuthTokenClaims, PlayersError>;
}

/// Auth token representing a result of a successful sign in.
#[derive(Debug, Clone, Serialize, Deserialize, derive_more::AsRef, PartialEq)]
pub struct AuthToken<'a>(pub Cow<'a, str>);

impl AuthToken<'_> {
    #[cfg(test)]
    /// Returns a test [AuthToken].
    pub fn test() -> AuthToken<'static> {
        AuthToken(Cow::Owned("auth_token".into()))
    }
}

/// [AuthToken] claims shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenClaims {
    /// When the token expires.
    pub exp: i64,

    /// When the token was issued.
    pub iat: i64,

    /// Subject [PlayerId] the token is issued for.
    pub sub: PlayerId,
}

impl AuthTokenClaims {
    #[cfg(test)]
    /// Returns a test [AuthTokenClaims].
    pub fn test() -> AuthTokenClaims {
        AuthTokenClaims {
            exp: 123,
            iat: 456,
            sub: PlayerId::test(),
        }
    }
}

impl<S: AppState> FromRequestParts<S> for AuthTokenClaims {
    type Rejection = PlayersError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header_value| header_value.to_str().ok())
        {
            Some(header_value_str) => {
                let jwt_token = AuthToken(Cow::Borrowed(&header_value_str["Bearer ".len()..]));
                let claims = state.jwt_service().verify_token(&jwt_token)?;
                Ok(claims)
            }

            None => Err(PlayersError::AuthTokenMissing),
        }
    }
}

/// Default implementation of [JwtService].
#[derive(Clone)]
pub struct JwtServiceDefault {
    /// JWT [Validation] settings.
    validation: jsonwebtoken::Validation,

    /// JWT [DecodingKey] for signature verification.
    decoding_key: jsonwebtoken::DecodingKey,

    /// JWT [EncodingKey] for signing.
    encoding_key: jsonwebtoken::EncodingKey,

    /// JWT [Header] for JWT creation.
    header: jsonwebtoken::Header,

    /// [Duration] how long the token is valid for.
    token_ttl: Duration,
}

impl JwtServiceDefault {
    /// Creates a new [JwtServiceDefault] with the given [JwtConfig].
    pub fn new(config: JwtConfig) -> JwtServiceDefault {
        let validation = jsonwebtoken::Validation::default();
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(config.secret.as_ref());
        let encoding_key = jsonwebtoken::EncodingKey::from_secret(config.secret.as_ref());
        let header = jsonwebtoken::Header::default();
        let token_ttl = config.ttl;

        JwtServiceDefault {
            validation,
            decoding_key,
            encoding_key,
            header,
            token_ttl,
        }
    }

    #[cfg(test)]
    /// Returns a test [JwtServiceDefault].
    pub fn test() -> JwtServiceDefault {
        JwtServiceDefault::new(JwtConfig::test())
    }
}

impl JwtService for JwtServiceDefault {
    fn create_token(&self, player_id: PlayerId) -> Result<AuthToken<'static>, PlayersError> {
        let now = Utc::now();

        let claims = AuthTokenClaims {
            exp: (now + self.token_ttl).timestamp(),
            iat: now.timestamp(),
            sub: player_id,
        };

        let token = jsonwebtoken::encode(&self.header, &claims, &self.encoding_key)
            .context("create auth token")?;

        Ok(AuthToken(Cow::Owned(token)))
    }

    fn verify_token(&self, token: &AuthToken) -> Result<AuthTokenClaims, PlayersError> {
        let claims = jsonwebtoken::decode(token.as_ref(), &self.decoding_key, &self.validation)
            .map_err(PlayersError::AuthToken)?
            .claims;

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_token_claims_json_snapshot() {
        insta::assert_json_snapshot!(&AuthTokenClaims::test());
    }

    #[test]
    fn jwt_service_encode_and_decode_succeeds() -> anyhow::Result<()> {
        let player_id = PlayerId::test();
        let service = JwtServiceDefault::test();
        let ttl = service.token_ttl;

        let token = service.create_token(player_id)?;
        let claims = service.verify_token(&token)?;

        assert_eq!(claims.sub, player_id);
        assert!(claims.iat <= Utc::now().timestamp());
        assert_eq!(claims.iat + ttl.num_seconds(), claims.exp);

        Ok(())
    }

    #[test]
    fn jwt_service_decode_rejects_expired_tokens() -> anyhow::Result<()> {
        let service = JwtServiceDefault::test();
        let now = Utc::now();
        let exp_in_past = now.timestamp() - service.validation.leeway as i64 - 10;
        let iat_in_past = exp_in_past - service.token_ttl.num_seconds();
        let claims_expired = AuthToken(
            jsonwebtoken::encode(
                &service.header,
                &AuthTokenClaims {
                    exp: exp_in_past,
                    iat: iat_in_past,
                    sub: PlayerId::test(),
                },
                &service.encoding_key,
            )?
            .into(),
        );

        let result = service.verify_token(&claims_expired);

        assert!(
            matches!(result, Err(PlayersError::AuthToken(e)) if *e.kind() == jsonwebtoken::errors::ErrorKind::ExpiredSignature)
        );

        Ok(())
    }

    #[test]
    fn jwt_service_decode_rejects_wrong_signatures() -> anyhow::Result<()> {
        let service = JwtServiceDefault::test();
        let service_with_different_secret = {
            let mut service = service.clone();
            service.encoding_key =
                jsonwebtoken::EncodingKey::from_secret("wrong-secret".as_bytes());
            service
        };

        let claims_wrong_signature = service_with_different_secret.create_token(PlayerId::test())?;

        let result = service.verify_token(&claims_wrong_signature);
        assert!(
            matches!(result, Err(PlayersError::AuthToken(e)) if *e.kind() == jsonwebtoken::errors::ErrorKind::InvalidSignature)
        );

        Ok(())
    }
}
