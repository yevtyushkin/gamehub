use serde::{Deserialize, Serialize};

/// Supported sign-in methods.
#[derive(Debug, Clone, PartialEq)]
pub enum SignInMethod {
    /// Sign in method using third party.
    ThirdParty(ThirdPartySignInMethod),
}

impl SignInMethod {
    #[cfg(test)]
    /// Returns a test [SignInMethod].
    pub fn test_google() -> SignInMethod {
        SignInMethod::ThirdParty(ThirdPartySignInMethod {
            provider: ThirdPartySignInProvider::Google,
            user_id: ThirdPartySignInUserId::test(),
        })
    }
}

/// Third-party sign-in method.
#[derive(Debug, Clone, PartialEq)]
pub struct ThirdPartySignInMethod {
    /// [ThirdPartySignInProvider] of this third-party sign-in method.
    pub provider: ThirdPartySignInProvider,

    /// User ID within the [ThirdPartySignInProvider].
    pub user_id: ThirdPartySignInUserId,
}

/// Supported [ThirdPartySignInMethod] providers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "third_party_sign_in_provider")]
pub enum ThirdPartySignInProvider {
    /// Google.
    Google,
}

/// User ID within the [ThirdPartySignInProvider].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
pub struct ThirdPartySignInUserId(pub String);

impl ThirdPartySignInUserId {
    #[cfg(test)]
    /// Returns a test [ThirdPartySignInUserId].
    pub fn test() -> ThirdPartySignInUserId {
        ThirdPartySignInUserId("test-user-id".into())
    }
}
