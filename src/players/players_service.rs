use crate::players::error::*;
use crate::players::http::*;
use crate::players::jwt_service::*;
use crate::players::player::*;
use crate::players::players_db::*;
use crate::players::sign_in_method::*;
use id_token_verifier::*;
use serde::*;
use sqlx::PgPool;

/// Provides logic working with [Player]s.
#[cfg_attr(test, mockall::automock)]
pub trait PlayersService {
    /// Handles the given [SignInRequest].
    fn sign_in(
        &self,
        request: &SignInRequest,
    ) -> impl Future<Output = Result<AuthToken<'static>, PlayersError>> + Send;

    /// Returns a [Player] by the given [PlayerId].
    fn player_by_id(
        &self,
        player_id: &PlayerId,
    ) -> impl Future<Output = Result<Player, PlayersError>> + Send;
}

/// Default [PlayersService] implementation.
pub struct PlayersServiceDefault<D = PgPool, GV = IdTokenVerifierDefault, JS = JwtServiceDefault> {
    /// [PlayersDb] for [Player]s db operations.
    players_db: D,

    /// [IdTokenVerifier] implementation for [ThirdPartySignInProvider::Google] ID tokens.
    google_id_token_verifier: GV,

    /// [JwtService] implementation to work with [Player]s JWT tokens.
    jwt_service: JS,
}

impl PlayersServiceDefault {
    /// Creates a new [PlayersServiceDefault] with the given [PgPool], Google
    /// [IdTokenVerifierConfig] and [JwtServiceDefault].
    pub fn new(
        pg_pool: PgPool,
        http_client: reqwest::Client,
        google_id_token_verifier_config: IdTokenVerifierConfig,
        jwt_service: JwtServiceDefault,
    ) -> PlayersServiceDefault {
        let players_db = pg_pool;
        let google_id_token_verifier =
            IdTokenVerifierDefault::new(google_id_token_verifier_config, http_client);

        PlayersServiceDefault {
            players_db,
            google_id_token_verifier,
            jwt_service,
        }
    }
}

impl<D, GV, JS> PlayersService for PlayersServiceDefault<D, GV, JS>
where
    D: PlayersDb + Sync,
    GV: IdTokenVerifier + Sync,
    JS: JwtService + Sync,
{
    async fn sign_in(&self, request: &SignInRequest) -> Result<AuthToken<'static>, PlayersError> {
        let sign_in_method = match request {
            SignInRequest::Google { id_token } => {
                let claims = self
                    .google_id_token_verifier
                    .verify::<ThirdPartyIdTokenClaims>(id_token.as_ref())
                    .await?;

                SignInMethod::ThirdParty(ThirdPartySignInMethod {
                    provider: ThirdPartySignInProvider::Google,
                    user_id: claims.sub,
                })
            }
        };

        let player = match self
            .players_db
            .find_player_with_sign_in_method(&sign_in_method)
            .await
        {
            Ok(player) => player,
            Err(PlayersError::PlayerNotFound) => {
                let player = Player {
                    id: PlayerId::random(),
                    screen_name: PlayerScreenName::random(),
                    joined_at: PlayerJoinedAt::now(),
                };

                self.players_db
                    .create_player_with_sign_in_method(&player, &sign_in_method)
                    .await?;

                player
            }
            e => e?,
        };

        let auth_token = self.jwt_service.create_token(player.id)?;

        Ok(auth_token)
    }

    async fn player_by_id(&self, player_id: &PlayerId) -> Result<Player, PlayersError> {
        self.players_db.find_player_by_id(player_id).await
    }
}

/// Target ID token claims. Used with [IdTokenVerifier::verify] when signing in with third party
/// sign in providers.
#[derive(Deserialize, Serialize)]
struct ThirdPartyIdTokenClaims {
    /// Subject [ThirdPartySignInUserId] the ID token is issued for.
    sub: ThirdPartySignInUserId,
}

impl ThirdPartyIdTokenClaims {
    #[cfg(test)]
    /// Returns a test [ThirdPartyIdTokenClaims].
    fn test() -> ThirdPartyIdTokenClaims {
        ThirdPartyIdTokenClaims {
            sub: ThirdPartySignInUserId::test(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use id_token_verifier::validation::ValidationError;
    use mockall::mock;
    use mockall::predicate::eq;
    use serde::de::DeserializeOwned;

    mock! {
        IdTokenVerifier {
            fn verify<T: DeserializeOwned + 'static>(
                &self,
                id_token: &str
            ) -> impl Future<Output = Result<T, IdTokenVerifierError>> + Send;
        }
    }
    impl IdTokenVerifier for MockIdTokenVerifier {
        fn verify<T: DeserializeOwned + 'static>(
            &self,
            id_token: &str,
        ) -> impl Future<Output = Result<T, IdTokenVerifierError>> + Send {
            self.verify(id_token)
        }
    }

    #[test]
    fn third_party_id_token_claims_json_snapshot() {
        insta::assert_json_snapshot!(&ThirdPartyIdTokenClaims::test());
    }

    #[tokio::test]
    async fn players_service_sign_in_fails_if_google_id_token_verification_fails() {
        let mut google_id_token_verifier = MockIdTokenVerifier::new();
        google_id_token_verifier
            .expect_verify::<ThirdPartyIdTokenClaims>()
            .withf(|id_token| id_token == IdToken::test().as_str())
            .returning(|_| {
                Box::pin(async {
                    Err(IdTokenVerifierError::Validation(
                        ValidationError::MissingKeyId,
                    ))
                })
            });

        let service = PlayersServiceDefault {
            google_id_token_verifier,
            players_db: MockPlayersDb::new(),
            jwt_service: MockJwtService::new(),
        };

        let result = service.sign_in(&SignInRequest::test_google()).await;

        assert!(matches!(
            result,
            Err(PlayersError::IdToken(IdTokenVerifierError::Validation(
                ValidationError::MissingKeyId,
            )))
        ));
    }

    #[tokio::test]
    async fn players_service_sign_in_fails_if_db_player_lookup_fails() {
        let mut google_id_token_verifier = MockIdTokenVerifier::new();
        google_id_token_verifier
            .expect_verify::<ThirdPartyIdTokenClaims>()
            .withf(|id_token| id_token == IdToken::test().as_str())
            .returning(|_| Box::pin(async { Ok(ThirdPartyIdTokenClaims::test()) }));

        let mut players_db = MockPlayersDb::new();
        players_db
            .expect_find_player_with_sign_in_method()
            .with(eq(SignInMethod::test_google()))
            .returning(|_| {
                Box::pin(async { Err(PlayersError::Internal(anyhow::anyhow!("oops"))) })
            });

        let service = PlayersServiceDefault {
            players_db,
            google_id_token_verifier,
            jwt_service: MockJwtService::new(),
        };

        let result = service.sign_in(&SignInRequest::test_google()).await;

        assert!(matches!(result, Err(PlayersError::Internal(e)) if e.to_string() == "oops"));
    }

    #[tokio::test]
    async fn players_service_sign_in_creates_player_if_player_does_not_exist() {
        let mut google_id_token_verifier = MockIdTokenVerifier::new();
        google_id_token_verifier
            .expect_verify::<ThirdPartyIdTokenClaims>()
            .withf(|id_token| id_token == IdToken::test().as_str())
            .returning(|_| Box::pin(async { Ok(ThirdPartyIdTokenClaims::test()) }));

        let mut players_db = MockPlayersDb::new();
        players_db
            .expect_find_player_with_sign_in_method()
            .with(eq(SignInMethod::test_google()))
            .returning(|_| Box::pin(async { Err(PlayersError::PlayerNotFound) }));
        players_db
            .expect_create_player_with_sign_in_method()
            .with(
                mockall::predicate::always(),
                eq(SignInMethod::test_google()),
            )
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut jwt_service = MockJwtService::new();
        jwt_service
            .expect_create_token()
            .returning(|_| Ok(AuthToken::test()));

        let service = PlayersServiceDefault {
            players_db,
            google_id_token_verifier,
            jwt_service,
        };

        let auth_token = service
            .sign_in(&SignInRequest::test_google())
            .await
            .unwrap();

        assert_eq!(auth_token, AuthToken::test());
    }

    #[tokio::test]
    async fn players_service_sign_in_creates_auth_token_from_existing_player() {
        let mut google_id_token_verifier = MockIdTokenVerifier::new();
        google_id_token_verifier
            .expect_verify::<ThirdPartyIdTokenClaims>()
            .withf(|id_token| id_token == IdToken::test().as_str())
            .returning(|_| Box::pin(async { Ok(ThirdPartyIdTokenClaims::test()) }));

        let mut players_db = MockPlayersDb::new();
        players_db
            .expect_find_player_with_sign_in_method()
            .with(eq(SignInMethod::test_google()))
            .returning(|_| Box::pin(async { Ok(Player::test()) }));

        let mut jwt_service = MockJwtService::new();
        jwt_service
            .expect_create_token()
            .with(eq(PlayerId::test()))
            .returning(|_| Ok(AuthToken::test()));

        let service = PlayersServiceDefault {
            players_db,
            google_id_token_verifier,
            jwt_service,
        };

        let auth_token = service
            .sign_in(&SignInRequest::test_google())
            .await
            .unwrap();

        assert_eq!(auth_token, AuthToken::test());
    }

    #[tokio::test]
    async fn players_service_player_by_id_lookups_player_in_db() {
        let mut players_db = MockPlayersDb::new();
        players_db
            .expect_find_player_by_id()
            .with(eq(PlayerId::test()))
            .returning(|_| Box::pin(async { Ok(Player::test()) }));

        let service = PlayersServiceDefault {
            players_db,
            google_id_token_verifier: MockIdTokenVerifier::new(),
            jwt_service: MockJwtService::new(),
        };

        let player = service.player_by_id(&PlayerId::test()).await.unwrap();

        assert_eq!(player, Player::test());
    }
}
