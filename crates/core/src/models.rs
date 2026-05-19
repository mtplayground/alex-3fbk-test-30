use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
pub struct MediaAssetId(Uuid);

impl MediaAssetId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for MediaAssetId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for MediaAssetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaJobId(Uuid);

impl MediaJobId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for MediaJobId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for MediaJobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(Uuid);

impl ConversationId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for ConversationId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for ConversationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for MessageId {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for MessageId {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Video,
}

impl MediaKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "image" => Some(Self::Image),
            "video" => Some(Self::Video),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaAssetStatus {
    Pending,
    Uploaded,
    Processing,
    Ready,
    Failed,
}

impl MediaAssetStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Uploaded => "uploaded",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "uploaded" => Some(Self::Uploaded),
            "processing" => Some(Self::Processing),
            "ready" => Some(Self::Ready),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaJobKind {
    ImageProcessing,
    VideoProcessing,
}

impl MediaJobKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImageProcessing => "image_processing",
            Self::VideoProcessing => "video_processing",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "image_processing" => Some(Self::ImageProcessing),
            "video_processing" => Some(Self::VideoProcessing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl MediaJobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationKind {
    Dm,
    Group,
}

impl ConversationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dm => "dm",
            Self::Group => "group",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "dm" => Some(Self::Dm),
            "group" => Some(Self::Group),
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
    is_admin: bool,
    suspended_at: Option<DateTime<Utc>>,
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
        is_admin: bool,
        suspended_at: Option<DateTime<Utc>>,
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
            is_admin,
            suspended_at,
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

    pub const fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn suspended_at(&self) -> Option<&DateTime<Utc>> {
        self.suspended_at.as_ref()
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateUserProfile {
    display_name: Option<String>,
    bio: Option<String>,
    link: Option<Option<String>>,
    is_private: Option<bool>,
}

impl UpdateUserProfile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_bio(mut self, bio: impl Into<String>) -> Self {
        self.bio = Some(bio.into());
        self
    }

    pub fn with_link(mut self, link: Option<String>) -> Self {
        self.link = Some(link);
        self
    }

    pub const fn with_is_private(mut self, is_private: bool) -> Self {
        self.is_private = Some(is_private);
        self
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }

    pub fn link(&self) -> Option<Option<&str>> {
        self.link.as_ref().map(|value| value.as_deref())
    }

    pub const fn is_private(&self) -> Option<bool> {
        self.is_private
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaAsset {
    id: MediaAssetId,
    owner_id: UserId,
    kind: MediaKind,
    status: MediaAssetStatus,
    original_key: String,
    variants: Value,
    duration_ms: Option<i64>,
    width: Option<i32>,
    height: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl MediaAsset {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MediaAssetId,
        owner_id: UserId,
        kind: MediaKind,
        status: MediaAssetStatus,
        original_key: String,
        variants: Value,
        duration_ms: Option<i64>,
        width: Option<i32>,
        height: Option<i32>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            owner_id,
            kind,
            status,
            original_key,
            variants,
            duration_ms,
            width,
            height,
            created_at,
            updated_at,
        }
    }

    pub const fn id(&self) -> MediaAssetId {
        self.id
    }

    pub const fn owner_id(&self) -> UserId {
        self.owner_id
    }

    pub const fn kind(&self) -> MediaKind {
        self.kind
    }

    pub const fn status(&self) -> MediaAssetStatus {
        self.status
    }

    pub fn original_key(&self) -> &str {
        &self.original_key
    }

    pub const fn variants(&self) -> &Value {
        &self.variants
    }

    pub const fn duration_ms(&self) -> Option<i64> {
        self.duration_ms
    }

    pub const fn width(&self) -> Option<i32> {
        self.width
    }

    pub const fn height(&self) -> Option<i32> {
        self.height
    }

    pub const fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub const fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMediaAsset {
    owner_id: UserId,
    kind: MediaKind,
    original_key: String,
    variants: Value,
}

impl CreateMediaAsset {
    pub fn new(owner_id: UserId, kind: MediaKind, original_key: impl Into<String>) -> Self {
        Self {
            owner_id,
            kind,
            original_key: original_key.into(),
            variants: Value::Object(Default::default()),
        }
    }

    pub fn with_variants(mut self, variants: Value) -> Self {
        self.variants = variants;
        self
    }

    pub const fn owner_id(&self) -> UserId {
        self.owner_id
    }

    pub const fn kind(&self) -> MediaKind {
        self.kind
    }

    pub fn original_key(&self) -> &str {
        &self.original_key
    }

    pub const fn variants(&self) -> &Value {
        &self.variants
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaJob {
    id: MediaJobId,
    asset_id: MediaAssetId,
    kind: MediaJobKind,
    status: MediaJobStatus,
    payload: Value,
    attempts: i32,
    max_attempts: i32,
    run_after: DateTime<Utc>,
    locked_at: Option<DateTime<Utc>>,
    locked_by: Option<String>,
    last_error: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl MediaJob {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MediaJobId,
        asset_id: MediaAssetId,
        kind: MediaJobKind,
        status: MediaJobStatus,
        payload: Value,
        attempts: i32,
        max_attempts: i32,
        run_after: DateTime<Utc>,
        locked_at: Option<DateTime<Utc>>,
        locked_by: Option<String>,
        last_error: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            asset_id,
            kind,
            status,
            payload,
            attempts,
            max_attempts,
            run_after,
            locked_at,
            locked_by,
            last_error,
            created_at,
            updated_at,
        }
    }

    pub const fn id(&self) -> MediaJobId {
        self.id
    }

    pub const fn asset_id(&self) -> MediaAssetId {
        self.asset_id
    }

    pub const fn kind(&self) -> MediaJobKind {
        self.kind
    }

    pub const fn status(&self) -> MediaJobStatus {
        self.status
    }

    pub const fn payload(&self) -> &Value {
        &self.payload
    }

    pub const fn attempts(&self) -> i32 {
        self.attempts
    }

    pub const fn max_attempts(&self) -> i32 {
        self.max_attempts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateMediaJob {
    asset_id: MediaAssetId,
    kind: MediaJobKind,
    payload: Value,
    max_attempts: i32,
    run_after: Option<DateTime<Utc>>,
}

impl CreateMediaJob {
    pub fn new(asset_id: MediaAssetId, kind: MediaJobKind) -> Self {
        Self {
            asset_id,
            kind,
            payload: Value::Object(Default::default()),
            max_attempts: 3,
            run_after: None,
        }
    }

    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub const fn asset_id(&self) -> MediaAssetId {
        self.asset_id
    }

    pub const fn kind(&self) -> MediaJobKind {
        self.kind
    }

    pub const fn payload(&self) -> &Value {
        &self.payload
    }

    pub const fn max_attempts(&self) -> i32 {
        self.max_attempts
    }

    pub const fn run_after(&self) -> Option<&DateTime<Utc>> {
        self.run_after.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    id: ConversationId,
    kind: ConversationKind,
    title: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new(
        id: ConversationId,
        kind: ConversationKind,
        title: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            kind,
            title,
            created_at,
            updated_at,
        }
    }

    pub const fn id(&self) -> ConversationId {
        self.id
    }

    pub const fn kind(&self) -> ConversationKind {
        self.kind
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub const fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub const fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateConversation {
    kind: ConversationKind,
    title: Option<String>,
}

impl CreateConversation {
    pub const fn new(kind: ConversationKind) -> Self {
        Self { kind, title: None }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub const fn kind(&self) -> ConversationKind {
        self.kind
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationMember {
    conversation_id: ConversationId,
    user_id: UserId,
    joined_at: DateTime<Utc>,
    last_read_message_id: Option<MessageId>,
}

impl ConversationMember {
    pub fn new(
        conversation_id: ConversationId,
        user_id: UserId,
        joined_at: DateTime<Utc>,
        last_read_message_id: Option<MessageId>,
    ) -> Self {
        Self {
            conversation_id,
            user_id,
            joined_at,
            last_read_message_id,
        }
    }

    pub const fn conversation_id(&self) -> ConversationId {
        self.conversation_id
    }

    pub const fn user_id(&self) -> UserId {
        self.user_id
    }

    pub const fn joined_at(&self) -> &DateTime<Utc> {
        &self.joined_at
    }

    pub const fn last_read_message_id(&self) -> Option<MessageId> {
        self.last_read_message_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    id: MessageId,
    conversation_id: ConversationId,
    author_id: UserId,
    body: String,
    media_id: Option<MediaAssetId>,
    created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(
        id: MessageId,
        conversation_id: ConversationId,
        author_id: UserId,
        body: String,
        media_id: Option<MediaAssetId>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            conversation_id,
            author_id,
            body,
            media_id,
            created_at,
        }
    }

    pub const fn id(&self) -> MessageId {
        self.id
    }

    pub const fn conversation_id(&self) -> ConversationId {
        self.conversation_id
    }

    pub const fn author_id(&self) -> UserId {
        self.author_id
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub const fn media_id(&self) -> Option<MediaAssetId> {
        self.media_id
    }

    pub const fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateMessage {
    conversation_id: ConversationId,
    author_id: UserId,
    body: String,
    media_id: Option<MediaAssetId>,
}

impl CreateMessage {
    pub fn new(conversation_id: ConversationId, author_id: UserId, body: impl Into<String>) -> Self {
        Self {
            conversation_id,
            author_id,
            body: body.into(),
            media_id: None,
        }
    }

    pub const fn with_media_id(mut self, media_id: MediaAssetId) -> Self {
        self.media_id = Some(media_id);
        self
    }

    pub const fn conversation_id(&self) -> ConversationId {
        self.conversation_id
    }

    pub const fn author_id(&self) -> UserId {
        self.author_id
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub const fn media_id(&self) -> Option<MediaAssetId> {
        self.media_id
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

    #[test]
    fn media_kind_round_trips_storage_value() {
        assert_eq!(
            MediaKind::from_str(MediaKind::Image.as_str()),
            Some(MediaKind::Image)
        );
        assert_eq!(
            MediaKind::from_str(MediaKind::Video.as_str()),
            Some(MediaKind::Video)
        );
        assert_eq!(MediaKind::from_str("audio"), None);
    }

    #[test]
    fn conversation_kind_round_trips_storage_value() {
        assert_eq!(
            ConversationKind::from_str(ConversationKind::Dm.as_str()),
            Some(ConversationKind::Dm)
        );
        assert_eq!(
            ConversationKind::from_str(ConversationKind::Group.as_str()),
            Some(ConversationKind::Group)
        );
        assert_eq!(ConversationKind::from_str("thread"), None);
    }
}
