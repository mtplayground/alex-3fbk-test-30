use argon2::password_hash::{
    Error as PasswordHashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm as JwtAlgorithm, DecodingKey, EncodingKey, Header, Validation,
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::models::UserId;

const ACCESS_TOKEN_MINUTES: i64 = 15;
const REFRESH_TOKEN_DAYS: i64 = 30;
const OPAQUE_TOKEN_BYTES: usize = 32;

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("password hash error: {0}")]
    PasswordHash(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("JWT subject is not a valid user id: {0}")]
    InvalidSubject(#[from] uuid::Error),

    #[error("JWT token type mismatch: expected {expected:?}, got {actual:?}")]
    TokenKindMismatch {
        expected: TokenKind,
        actual: TokenKind,
    },

    #[error("token timestamp is outside the supported range")]
    TimestampOutOfRange,
}

impl From<PasswordHashError> for AuthError {
    fn from(error: PasswordHashError) -> Self {
        Self::PasswordHash(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedToken {
    token: String,
    claims: JwtClaims,
}

impl SignedToken {
    pub fn new(token: String, claims: JwtClaims) -> Self {
        Self { token, claims }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub const fn claims(&self) -> &JwtClaims {
        &self.claims
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Access,
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwtClaims {
    sub: String,
    typ: TokenKind,
    exp: usize,
    iat: usize,
    jti: Option<Uuid>,
}

impl JwtClaims {
    pub fn new(
        user_id: UserId,
        token_kind: TokenKind,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        jti: Option<Uuid>,
    ) -> Result<Self> {
        Ok(Self {
            sub: user_id.to_string(),
            typ: token_kind,
            exp: timestamp_to_usize(expires_at)?,
            iat: timestamp_to_usize(issued_at)?,
            jti,
        })
    }

    pub fn subject(&self) -> &str {
        &self.sub
    }

    pub fn user_id(&self) -> Result<UserId> {
        Ok(UserId::from(Uuid::parse_str(&self.sub)?))
    }

    pub const fn token_kind(&self) -> TokenKind {
        self.typ
    }

    pub const fn expires_at_timestamp(&self) -> usize {
        self.exp
    }

    pub fn expires_at(&self) -> Result<DateTime<Utc>> {
        let timestamp = i64::try_from(self.exp).map_err(|_| AuthError::TimestampOutOfRange)?;
        DateTime::from_timestamp(timestamp, 0).ok_or(AuthError::TimestampOutOfRange)
    }

    pub const fn issued_at_timestamp(&self) -> usize {
        self.iat
    }

    pub const fn jti(&self) -> Option<Uuid> {
        self.jti
    }
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2id();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    let argon2 = argon2id();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(PasswordHashError::Password) => Ok(false),
        Err(error) => Err(AuthError::PasswordHash(error.to_string())),
    }
}

pub fn generate_opaque_token() -> String {
    let mut bytes = [0_u8; OPAQUE_TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_opaque_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

pub fn sign_access_token(jwt: &JwtConfig, user_id: UserId) -> Result<SignedToken> {
    let now = Utc::now();
    let expires_at = access_token_expires_at(now);
    sign_token(jwt, user_id, TokenKind::Access, now, expires_at, None)
}

pub fn sign_refresh_token(jwt: &JwtConfig, user_id: UserId) -> Result<SignedToken> {
    let now = Utc::now();
    let expires_at = refresh_token_expires_at(now);
    sign_token(
        jwt,
        user_id,
        TokenKind::Refresh,
        now,
        expires_at,
        Some(Uuid::new_v4()),
    )
}

pub fn verify_access_token(jwt: &JwtConfig, token: &str) -> Result<JwtClaims> {
    verify_token(jwt, token, TokenKind::Access)
}

pub fn verify_refresh_token(jwt: &JwtConfig, token: &str) -> Result<JwtClaims> {
    verify_token(jwt, token, TokenKind::Refresh)
}

pub fn access_token_expires_at(issued_at: DateTime<Utc>) -> DateTime<Utc> {
    issued_at + Duration::minutes(ACCESS_TOKEN_MINUTES)
}

pub fn refresh_token_expires_at(issued_at: DateTime<Utc>) -> DateTime<Utc> {
    issued_at + Duration::days(REFRESH_TOKEN_DAYS)
}

fn sign_token(
    jwt: &JwtConfig,
    user_id: UserId,
    token_kind: TokenKind,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    jti: Option<Uuid>,
) -> Result<SignedToken> {
    let claims = JwtClaims::new(user_id, token_kind, issued_at, expires_at, jti)?;
    let header = Header::new(JwtAlgorithm::HS256);
    let key = EncodingKey::from_secret(jwt.secret().as_bytes());
    let token = encode(&header, &claims, &key)?;

    Ok(SignedToken::new(token, claims))
}

fn verify_token(jwt: &JwtConfig, token: &str, expected_kind: TokenKind) -> Result<JwtClaims> {
    let key = DecodingKey::from_secret(jwt.secret().as_bytes());
    let validation = Validation::new(JwtAlgorithm::HS256);
    let claims = decode::<JwtClaims>(token, &key, &validation)?.claims;

    if claims.token_kind() != expected_kind {
        return Err(AuthError::TokenKindMismatch {
            expected: expected_kind,
            actual: claims.token_kind(),
        });
    }

    Ok(claims)
}

fn argon2id() -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())
}

fn timestamp_to_usize(timestamp: DateTime<Utc>) -> Result<usize> {
    usize::try_from(timestamp.timestamp()).map_err(|_| AuthError::TimestampOutOfRange)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_jwt_config() -> JwtConfig {
        JwtConfig::from_secret("test-secret-with-enough-entropy")
    }

    #[test]
    fn password_hash_round_trip_verifies_only_original_password() {
        let hash = hash_password("correct horse battery staple").expect("hash should be created");

        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password("correct horse battery staple", &hash).expect("hash should verify"));
        assert!(!verify_password("wrong password", &hash).expect("hash should verify"));
    }

    #[test]
    fn opaque_token_hash_is_stable_and_not_plaintext() {
        let token = "opaque-token";
        let hash = hash_opaque_token(token);

        assert_eq!(hash, hash_opaque_token(token));
        assert_ne!(hash, token);
    }

    #[test]
    fn access_token_round_trip_contains_user_id() {
        let user_id = UserId::from(Uuid::new_v4());
        let signed = sign_access_token(&test_jwt_config(), user_id).expect("token should sign");
        let claims =
            verify_access_token(&test_jwt_config(), signed.token()).expect("token verifies");

        assert_eq!(claims.user_id().expect("subject should parse"), user_id);
        assert_eq!(claims.token_kind(), TokenKind::Access);
        assert_eq!(claims.jti(), None);
        assert_eq!(
            claims.expires_at_timestamp() - claims.issued_at_timestamp(),
            Duration::minutes(ACCESS_TOKEN_MINUTES).num_seconds() as usize
        );
    }

    #[test]
    fn refresh_token_round_trip_contains_rotation_jti() {
        let user_id = UserId::from(Uuid::new_v4());
        let signed = sign_refresh_token(&test_jwt_config(), user_id).expect("token should sign");
        let claims =
            verify_refresh_token(&test_jwt_config(), signed.token()).expect("token verifies");

        assert_eq!(claims.user_id().expect("subject should parse"), user_id);
        assert_eq!(claims.token_kind(), TokenKind::Refresh);
        assert!(claims.jti().is_some());
        assert_eq!(
            claims.expires_at_timestamp() - claims.issued_at_timestamp(),
            Duration::days(REFRESH_TOKEN_DAYS).num_seconds() as usize
        );
    }

    #[test]
    fn verifier_rejects_unexpected_token_kind() {
        let user_id = UserId::from(Uuid::new_v4());
        let signed = sign_refresh_token(&test_jwt_config(), user_id).expect("token should sign");
        let error = verify_access_token(&test_jwt_config(), signed.token())
            .expect_err("refresh token should not verify as access token");

        assert!(matches!(
            error,
            AuthError::TokenKindMismatch {
                expected: TokenKind::Access,
                actual: TokenKind::Refresh
            }
        ));
    }
}
