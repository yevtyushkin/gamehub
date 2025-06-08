use axum::http::StatusCode;

/// Healthcheck router.
pub fn router() -> axum::Router {
    axum::Router::new().route("/health", axum::routing::get(health))
}

/// Healthcheck handler.
pub async fn health() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health() -> anyhow::Result<()> {
        let router = TestServer::new(router())?;

        router.get("/health").await.assert_status(StatusCode::OK);

        Ok(())
    }
}
