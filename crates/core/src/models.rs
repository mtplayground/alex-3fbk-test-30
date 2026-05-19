use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RefreshTokenId(Uuid);

impl RefreshTokenId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for RefreshTokenId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for RefreshTokenId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthTokenId(Uuid);

impl AuthTokenId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for AuthTokenId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for AuthTokenId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthTokenPurpose {
    EmailVerification,
    PasswordReset,
}

impl AuthTokenPurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmailVerification => "email_verification",
            Self::PasswordReset => "password_reset",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "email_verification" => Some(Self::EmailVerification),
            "password_reset" => Some(Self::PasswordReset),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    email: String,
    handle: String,
    password_hash: String,
    display_name: String,
    bio: String,
    link: Option<String>,
    avatar_key: Option<String>,
    is_private: bool,
    email_verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl User {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: UserId,
        email: String,
        handle: String,
        password_hash: String,
        display_name: String,
        bio: String,
        link: Option<String>,
        avatar_key: Option<String>,
        is_private: bool,
        email_verified_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            handle,
            password_hash,
            display_name,
            bio,
            link,
            avatar_key,
            is_private,
            email_verified_at,
            created_at,
        }
    }

    pub const fn id(&self) -> UserId {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn bio(&self) -> &str {
        &self.bio
    }

    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    pub fn avatar_key(&self) -> Option<&str> {
        self.avatar_key.as_deref()
    }

    pub const fn is_private(&self) -> bool {
        self.is_private
    }

    pub fn email_verified_at(&self) -> Option<&DateTime<Utc>> {
        self.email_verified_at.as_ref()
    }

    pub const fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateUser {
    email: String,
    handle: String,
    password_hash: String,
    display_name: String,
    bio: String,
    link: Option<String>,
    avatar_key: Option<String>,
    is_private: bool,
}

impl CreateUser {
    pub fn new(
        email: impl Into<String>,
        handle: impl Into<String>,
        password_hash: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into(),
            handle: handle.into(),
            password_hash: password_hash.into(),
            display_name: display_name.into(),
            bio: String::new(),
            link: None,
            avatar_key: None,
            is_private: false,
        }
    }

    pub fn with_bio(mut self, bio: impl Into<String>) -> Self {
        self.bio = bio.into();
        self
    }

    pub fn with_link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    pub fn with_avatar_key(mut self, avatar_key: impl Into<String>) -> Self {
        self.avatar_key = Some(avatar_key.into());
        self
    }

    pub const fn private(mut self, is_private: bool) -> Self {
        self.is_private = is_private;
        self
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn normalized_email(&self) -> String {
        self.email.trim().to_ascii_lowercase()
    }

    pub fn handle(&self) -> &str {
        &self.handle
    }

    pub fn normalized_handle(&self) -> String {
        self.handle.trim().to_ascii_lowercase()
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn bio(&self) -> &str {
        &self.bio
    }

    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    pub fn avatar_key(&self) -> Option<&str> {
        self.avatar_key.as_deref()
    }

    pub const fn is_private(&self) -> bool {
        self.is_private
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    id: RefreshTokenId,
    user_id: UserId,
    token_jti: Uuid,
    rotated_from_token_id: Option<RefreshTokenId>,
    replaced_by_token_id: Option<RefreshTokenId>,
    revoked_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl RefreshToken {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RefreshTokenId,
        user_id: UserId,
        token_jti: Uuid,
        rotated_from_token_id: Option<RefreshTokenId>,
        replaced_by_token_id: Option<RefreshTokenId>,
        revoked_at: Option<DateTime<Utc>>,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            token_jti,
            rotated_from_token_id,
            replaced_by_token_id,
            revoked_at,
            expires_at,
            created_at,
        }
    }

    pub const fn id(&self) -> RefreshTokenId {
        self.id
    }

    pub const fn user_id(&self) -> UserId {
        self.user_id
    }

    pub const fn token_jti(&self) -> Uuid {
        self.token_jti
    }

    pub const fn rotated_from_token_id(&self) -> Option<RefreshTokenId> {
        self.rotated_from_token_id
    }

    pub const fn replaced_by_token_id(&self) -> Option<RefreshTokenId> {
        self.replaced_by_token_id
    }

    pub fn revoked_at(&self) -> Option<&DateTime<Utc>> {
        self.revoked_at.as_ref()
    }

    pub const fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    pub const fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.replaced_by_token_id.is_none() && self.expires_at > now
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRefreshToken {
    user_id: UserId,
    token_jti: Uuid,
    expires_at: DateTime<Utc>,
    rotated_from_token_id: Option<RefreshTokenId>,
}

impl CreateRefreshToken {
    pub fn new(user_id: UserId, token_jti: Uuid, expires_at: DateTime<Utc>) -> Self {
        Self {
            user_id,
            token_jti,
            expires_at,
            rotated_from_token_id: None,
        }
    }

    pub const fn rotated_from(mut self, token_id: RefreshTokenId) -> Self {
        self.rotated_from_token_id = Some(token_id);
        self
    }

    pub const fn user_id(&self) -> UserId {
        self.user_id
    }

    pub const fn token_jti(&self) -> Uuid {
        self.token_jti
    }

    pub const fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    pub const fn rotated_from_token_id(&self) -> Option<RefreshTokenId> {
        self.rotated_from_token_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthToken {
    id: AuthTokenId,
    user_id: UserId,
    token_hash: String,
    purpose: AuthTokenPurpose,
    consumed_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl AuthToken {
    pub fn new(
        id: AuthTokenId,
        user_id: UserId,
        token_hash: String,
        purpose: AuthTokenPurpose,
        consumed_at: Option<DateTime<Utc>>,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            token_hash,
            purpose,
            consumed_at,
            expires_at,
            created_at,
        }
    }

    pub const fn id(&self) -> AuthTokenId {
        self.id
    }

    pub const fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn token_hash(&self) -> &str {
        &self.token_hash
    }

    pub const fn purpose(&self) -> AuthTokenPurpose {
        self.purpose
    }

    pub fn consumed_at(&self) -> Option<&DateTime<Utc>> {
        self.consumed_at.as_ref()
    }

    pub const fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    pub const fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAuthToken {
    user_id: UserId,
    token_hash: String,
    purpose: AuthTokenPurpose,
    expires_at: DateTime<Utc>,
}

impl CreateAuthToken {
    pub fn new(
        user_id: UserId,
        token_hash: impl Into<String>,
        purpose: AuthTokenPurpose,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            token_hash: token_hash.into(),
            purpose,
            expires_at,
        }
    }

    pub const fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn token_hash(&self) -> &str {
        &self.token_hash
    }

    pub const fn purpose(&self) -> AuthTokenPurpose {
        self.purpose
    }

    pub const fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub name: String,
    pub version: String,
}

impl ServiceMetadata {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user_normalizes_lookup_fields() {
        let input = CreateUser::new(
            "  USER@example.COM ",
            "  ZeroClaw ",
            "password-hash",
            "Zero Claw",
        );

        assert_eq!(input.normalized_email(), "user@example.com");
        assert_eq!(input.normalized_handle(), "zeroclaw");
    }

    #[test]
    fn auth_token_purpose_round_trips_storage_value() {
        assert_eq!(
            AuthTokenPurpose::from_str(AuthTokenPurpose::EmailVerification.as_str()),
            Some(AuthTokenPurpose::EmailVerification)
        );
        assert_eq!(
            AuthTokenPurpose::from_str(AuthTokenPurpose::PasswordReset.as_str()),
            Some(AuthTokenPurpose::PasswordReset)
        );
        assert_eq!(AuthTokenPurpose::from_str("unknown"), None);
    }
}
