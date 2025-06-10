use crate::players::error::PlayersError;
use crate::players::player::*;
use crate::players::sign_in_method::*;
use anyhow::Context;
use sqlx::{PgPool, query, query_as};
use std::ops::DerefMut;

/// Defines db operations with [Player]s.
#[cfg_attr(test, mockall::automock)]
pub trait PlayersDb {
    /// Creates a new [Player] with the given [SignInMethod] in the database.
    fn create_player_with_sign_in_method(
        &self,
        player: &Player,
        sign_in_method: &SignInMethod,
    ) -> impl Future<Output = Result<(), PlayersError>> + Send;

    /// Finds a [Player] with the matching [SignInMethod] in the database.
    fn find_player_with_sign_in_method(
        &self,
        sign_in_method: &SignInMethod,
    ) -> impl Future<Output = Result<Player, PlayersError>> + Send;

    /// Finds a [Player] by the given [PlayerId] in the database.
    fn find_player_by_id(
        &self,
        player_id: &PlayerId,
    ) -> impl Future<Output = Result<Player, PlayersError>> + Send;
}

impl PlayersDb for PgPool {
    async fn create_player_with_sign_in_method(
        &self,
        player: &Player,
        sign_in_method: &SignInMethod,
    ) -> Result<(), PlayersError> {
        let mut tx = self.begin().await.context("begin transaction")?;

        query!(
            r#"
            insert into player (id, screen_name, joined_at)
            values ($1, $2, $3)
            "#,
            &player.id as &PlayerId,
            &player.screen_name as &PlayerScreenName,
            &player.joined_at as &PlayerJoinedAt
        )
        .execute(tx.deref_mut())
        .await
        .context("create player")?;

        match sign_in_method {
            SignInMethod::ThirdParty(third_party) => query!(
                r#"
                insert into third_party_sign_in_method (provider, user_id, player_id)
                values ($1, $2, $3)
                "#,
                &third_party.provider as &ThirdPartySignInProvider,
                &third_party.user_id as &ThirdPartySignInUserId,
                &player.id as &PlayerId
            )
            .execute(tx.deref_mut()),
        }
        .await
        .context("create sign in method")?;

        tx.commit().await.context("commit transaction")?;

        Ok(())
    }

    async fn find_player_with_sign_in_method(
        &self,
        sign_in_method: &SignInMethod,
    ) -> Result<Player, PlayersError> {
        let player = match sign_in_method {
            SignInMethod::ThirdParty(third_party) => query_as!(
                Player,
                r#"
                select
                    p.id as "id: PlayerId",
                    p.screen_name as "screen_name: PlayerScreenName",
                    p.joined_at as "joined_at: PlayerJoinedAt"
                from player p
                join third_party_sign_in_method t on p.id = t.player_id
                where t.provider = $1 and t.user_id = $2
                "#,
                &third_party.provider as &ThirdPartySignInProvider,
                &third_party.user_id as &ThirdPartySignInUserId
            )
            .fetch_optional(self),
        }
        .await
        .context("find player with sign in method")?
        .ok_or(PlayersError::PlayerNotFound)?;

        Ok(player)
    }

    async fn find_player_by_id(&self, player_id: &PlayerId) -> Result<Player, PlayersError> {
        query_as!(
            Player,
            r#"
            select
                id as "id: PlayerId",
                screen_name as "screen_name: PlayerScreenName",
                joined_at as "joined_at: PlayerJoinedAt"
            from player
            where id = $1
            "#,
            player_id as &PlayerId
        )
        .fetch_optional(self)
        .await
        .context("find player by id")?
        .ok_or(PlayersError::PlayerNotFound)
    }
}
