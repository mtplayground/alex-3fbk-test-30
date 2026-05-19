pub mod users {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::{CreateUser, UpdateUserProfile, User, UserId};

    #[derive(Debug, FromRow)]
    struct UserRow {
        id: Uuid,
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

    impl From<UserRow> for User {
        fn from(row: UserRow) -> Self {
            Self::new(
                UserId::from(row.id),
                row.email,
                row.handle,
                row.password_hash,
                row.display_name,
                row.bio,
                row.link,
                row.avatar_key,
                row.is_private,
                row.email_verified_at,
                row.created_at,
            )
        }
    }

    pub async fn create(pool: &PgPool, input: &CreateUser) -> sqlx::Result<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (
                email,
                handle,
                password_hash,
                display_name,
                bio,
                link,
                avatar_key,
                is_private
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
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
                created_at
            "#,
        )
        .bind(input.normalized_email())
        .bind(input.normalized_handle())
        .bind(input.password_hash())
        .bind(input.display_name())
        .bind(input.bio())
        .bind(input.link())
        .bind(input.avatar_key())
        .bind(input.is_private())
        .fetch_one(pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn find_by_id(pool: &PgPool, id: UserId) -> sqlx::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(SELECT_USER_SQL_WITH_ID)
            .bind(id.as_uuid())
            .fetch_optional(pool)
            .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
        let normalized = email.trim().to_ascii_lowercase();

        let row = sqlx::query_as::<_, UserRow>(SELECT_USER_SQL_WITH_EMAIL)
            .bind(normalized)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_by_handle(pool: &PgPool, handle: &str) -> sqlx::Result<Option<User>> {
        let normalized = handle.trim().to_ascii_lowercase();

        let row = sqlx::query_as::<_, UserRow>(SELECT_USER_SQL_WITH_HANDLE)
            .bind(normalized)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(User::from))
    }

    pub async fn mark_email_verified(pool: &PgPool, id: UserId) -> sqlx::Result<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET email_verified_at = COALESCE(email_verified_at, now())
            WHERE id = $1
            RETURNING
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
                created_at
            "#,
        )
        .bind(id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_password_hash(
        pool: &PgPool,
        id: UserId,
        password_hash: &str,
    ) -> sqlx::Result<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
            RETURNING
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
                created_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(password_hash)
        .fetch_one(pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_profile(
        pool: &PgPool,
        id: UserId,
        input: &UpdateUserProfile,
    ) -> sqlx::Result<User> {
        let link_value = input.link().flatten().map(str::to_owned);
        let should_update_link = input.link().is_some();

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET
                display_name = COALESCE($2, display_name),
                bio = COALESCE($3, bio),
                link = CASE WHEN $4 THEN $5 ELSE link END,
                is_private = COALESCE($6, is_private)
            WHERE id = $1
            RETURNING
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
                created_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.display_name())
        .bind(input.bio())
        .bind(should_update_link)
        .bind(link_value.as_deref())
        .bind(input.is_private())
        .fetch_one(pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_avatar_key(
        pool: &PgPool,
        id: UserId,
        avatar_key: &str,
    ) -> sqlx::Result<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET avatar_key = $2
            WHERE id = $1
            RETURNING
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
                created_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(avatar_key)
        .fetch_one(pool)
        .await?;

        Ok(User::from(row))
    }

    const SELECT_USER_SQL_WITH_ID: &str = r#"
        SELECT
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
            created_at
        FROM users
        WHERE id = $1
    "#;

    const SELECT_USER_SQL_WITH_EMAIL: &str = r#"
        SELECT
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
            created_at
        FROM users
        WHERE email = $1
    "#;

    const SELECT_USER_SQL_WITH_HANDLE: &str = r#"
        SELECT
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
            created_at
        FROM users
        WHERE handle = $1
    "#;
}

pub mod refresh_tokens {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::{CreateRefreshToken, RefreshToken, RefreshTokenId, UserId};

    #[derive(Debug, FromRow)]
    struct RefreshTokenRow {
        id: Uuid,
        user_id: Uuid,
        token_jti: Uuid,
        rotated_from_token_id: Option<Uuid>,
        replaced_by_token_id: Option<Uuid>,
        revoked_at: Option<DateTime<Utc>>,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    }

    impl From<RefreshTokenRow> for RefreshToken {
        fn from(row: RefreshTokenRow) -> Self {
            Self::new(
                RefreshTokenId::from(row.id),
                UserId::from(row.user_id),
                row.token_jti,
                row.rotated_from_token_id.map(RefreshTokenId::from),
                row.replaced_by_token_id.map(RefreshTokenId::from),
                row.revoked_at,
                row.expires_at,
                row.created_at,
            )
        }
    }

    pub async fn create(pool: &PgPool, input: &CreateRefreshToken) -> sqlx::Result<RefreshToken> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            INSERT INTO refresh_tokens (
                user_id,
                token_jti,
                rotated_from_token_id,
                expires_at
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                user_id,
                token_jti,
                rotated_from_token_id,
                replaced_by_token_id,
                revoked_at,
                expires_at,
                created_at
            "#,
        )
        .bind(input.user_id().as_uuid())
        .bind(input.token_jti())
        .bind(input.rotated_from_token_id().map(RefreshTokenId::as_uuid))
        .bind(input.expires_at())
        .fetch_one(pool)
        .await?;

        Ok(RefreshToken::from(row))
    }

    pub async fn rotate(
        pool: &PgPool,
        current_id: RefreshTokenId,
        input: &CreateRefreshToken,
    ) -> sqlx::Result<RefreshToken> {
        let mut transaction = pool.begin().await?;

        let replacement = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            INSERT INTO refresh_tokens (
                user_id,
                token_jti,
                rotated_from_token_id,
                expires_at
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                user_id,
                token_jti,
                rotated_from_token_id,
                replaced_by_token_id,
                revoked_at,
                expires_at,
                created_at
            "#,
        )
        .bind(input.user_id().as_uuid())
        .bind(input.token_jti())
        .bind(input.rotated_from_token_id().map(RefreshTokenId::as_uuid))
        .bind(input.expires_at())
        .fetch_one(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET
                replaced_by_token_id = $2,
                revoked_at = COALESCE(revoked_at, now())
            WHERE id = $1
            "#,
        )
        .bind(current_id.as_uuid())
        .bind(replacement.id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(RefreshToken::from(replacement))
    }

    pub async fn find_by_jti(pool: &PgPool, token_jti: Uuid) -> sqlx::Result<Option<RefreshToken>> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(SELECT_REFRESH_TOKEN_SQL_WITH_JTI)
            .bind(token_jti)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(RefreshToken::from))
    }

    pub async fn find_active_by_jti(
        pool: &PgPool,
        token_jti: Uuid,
    ) -> sqlx::Result<Option<RefreshToken>> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(SELECT_ACTIVE_REFRESH_TOKEN_SQL_WITH_JTI)
            .bind(token_jti)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(RefreshToken::from))
    }

    pub async fn mark_rotated(
        pool: &PgPool,
        current_id: RefreshTokenId,
        replacement_id: RefreshTokenId,
    ) -> sqlx::Result<RefreshToken> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            UPDATE refresh_tokens
            SET
                replaced_by_token_id = $2,
                revoked_at = COALESCE(revoked_at, now())
            WHERE id = $1
            RETURNING
                id,
                user_id,
                token_jti,
                rotated_from_token_id,
                replaced_by_token_id,
                revoked_at,
                expires_at,
                created_at
            "#,
        )
        .bind(current_id.as_uuid())
        .bind(replacement_id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(RefreshToken::from(row))
    }

    pub async fn revoke(pool: &PgPool, id: RefreshTokenId) -> sqlx::Result<RefreshToken> {
        let row = sqlx::query_as::<_, RefreshTokenRow>(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = COALESCE(revoked_at, now())
            WHERE id = $1
            RETURNING
                id,
                user_id,
                token_jti,
                rotated_from_token_id,
                replaced_by_token_id,
                revoked_at,
                expires_at,
                created_at
            "#,
        )
        .bind(id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(RefreshToken::from(row))
    }

    pub async fn revoke_all_for_user(pool: &PgPool, user_id: UserId) -> sqlx::Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = COALESCE(revoked_at, now())
            WHERE user_id = $1
                AND revoked_at IS NULL
            "#,
        )
        .bind(user_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    const SELECT_REFRESH_TOKEN_SQL_WITH_JTI: &str = r#"
        SELECT
            id,
            user_id,
            token_jti,
            rotated_from_token_id,
            replaced_by_token_id,
            revoked_at,
            expires_at,
            created_at
        FROM refresh_tokens
        WHERE token_jti = $1
    "#;

    const SELECT_ACTIVE_REFRESH_TOKEN_SQL_WITH_JTI: &str = r#"
        SELECT
            id,
            user_id,
            token_jti,
            rotated_from_token_id,
            replaced_by_token_id,
            revoked_at,
            expires_at,
            created_at
        FROM refresh_tokens
        WHERE token_jti = $1
            AND replaced_by_token_id IS NULL
            AND revoked_at IS NULL
            AND expires_at > now()
    "#;
}

pub mod auth_tokens {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::{AuthToken, AuthTokenId, AuthTokenPurpose, CreateAuthToken, UserId};

    #[derive(Debug, FromRow)]
    struct AuthTokenRow {
        id: Uuid,
        user_id: Uuid,
        token_hash: String,
        purpose: String,
        consumed_at: Option<DateTime<Utc>>,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    }

    impl TryFrom<AuthTokenRow> for AuthToken {
        type Error = sqlx::Error;

        fn try_from(row: AuthTokenRow) -> Result<Self, Self::Error> {
            let purpose = AuthTokenPurpose::from_str(&row.purpose).ok_or_else(|| {
                sqlx::Error::ColumnDecode {
                    index: "purpose".to_owned(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unknown auth token purpose {:?}", row.purpose),
                    )
                    .into(),
                }
            })?;

            Ok(Self::new(
                AuthTokenId::from(row.id),
                UserId::from(row.user_id),
                row.token_hash,
                purpose,
                row.consumed_at,
                row.expires_at,
                row.created_at,
            ))
        }
    }

    pub async fn create(pool: &PgPool, input: &CreateAuthToken) -> sqlx::Result<AuthToken> {
        let row = sqlx::query_as::<_, AuthTokenRow>(
            r#"
            INSERT INTO auth_tokens (
                user_id,
                token_hash,
                purpose,
                expires_at
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                user_id,
                token_hash,
                purpose,
                consumed_at,
                expires_at,
                created_at
            "#,
        )
        .bind(input.user_id().as_uuid())
        .bind(input.token_hash())
        .bind(input.purpose().as_str())
        .bind(input.expires_at())
        .fetch_one(pool)
        .await?;

        AuthToken::try_from(row)
    }

    pub async fn consume_active_by_hash(
        pool: &PgPool,
        purpose: AuthTokenPurpose,
        token_hash: &str,
    ) -> sqlx::Result<Option<AuthToken>> {
        let row = sqlx::query_as::<_, AuthTokenRow>(
            r#"
            UPDATE auth_tokens
            SET consumed_at = now()
            WHERE token_hash = $1
                AND purpose = $2
                AND consumed_at IS NULL
                AND expires_at > now()
            RETURNING
                id,
                user_id,
                token_hash,
                purpose,
                consumed_at,
                expires_at,
                created_at
            "#,
        )
        .bind(token_hash)
        .bind(purpose.as_str())
        .fetch_optional(pool)
        .await?;

        row.map(AuthToken::try_from).transpose()
    }
}

pub mod media {
    use chrono::{DateTime, Utc};
    use serde_json::Value;
    use sqlx::types::Json;
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::{
        CreateMediaAsset, CreateMediaJob, MediaAsset, MediaAssetId, MediaAssetStatus, MediaJob,
        MediaJobId, MediaJobKind, MediaJobStatus, MediaKind, UserId,
    };

    #[derive(Debug, FromRow)]
    struct MediaAssetRow {
        id: Uuid,
        owner_id: Uuid,
        kind: String,
        status: String,
        original_key: String,
        variants: Json<Value>,
        duration_ms: Option<i64>,
        width: Option<i32>,
        height: Option<i32>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    impl TryFrom<MediaAssetRow> for MediaAsset {
        type Error = sqlx::Error;

        fn try_from(row: MediaAssetRow) -> Result<Self, Self::Error> {
            let kind = MediaKind::from_str(&row.kind).ok_or_else(|| {
                decode_error("kind", format!("unknown media kind {:?}", row.kind))
            })?;
            let status = MediaAssetStatus::from_str(&row.status).ok_or_else(|| {
                decode_error(
                    "status",
                    format!("unknown media asset status {:?}", row.status),
                )
            })?;

            Ok(Self::new(
                MediaAssetId::from(row.id),
                UserId::from(row.owner_id),
                kind,
                status,
                row.original_key,
                row.variants.0,
                row.duration_ms,
                row.width,
                row.height,
                row.created_at,
                row.updated_at,
            ))
        }
    }

    #[derive(Debug, FromRow)]
    struct MediaJobRow {
        id: Uuid,
        asset_id: Uuid,
        kind: String,
        status: String,
        payload: Json<Value>,
        attempts: i32,
        max_attempts: i32,
        run_after: DateTime<Utc>,
        locked_at: Option<DateTime<Utc>>,
        locked_by: Option<String>,
        last_error: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    impl TryFrom<MediaJobRow> for MediaJob {
        type Error = sqlx::Error;

        fn try_from(row: MediaJobRow) -> Result<Self, Self::Error> {
            let kind = MediaJobKind::from_str(&row.kind).ok_or_else(|| {
                decode_error("kind", format!("unknown media job kind {:?}", row.kind))
            })?;
            let status = MediaJobStatus::from_str(&row.status).ok_or_else(|| {
                decode_error(
                    "status",
                    format!("unknown media job status {:?}", row.status),
                )
            })?;

            Ok(Self::new(
                MediaJobId::from(row.id),
                MediaAssetId::from(row.asset_id),
                kind,
                status,
                row.payload.0,
                row.attempts,
                row.max_attempts,
                row.run_after,
                row.locked_at,
                row.locked_by,
                row.last_error,
                row.created_at,
                row.updated_at,
            ))
        }
    }

    pub async fn create_asset(pool: &PgPool, input: &CreateMediaAsset) -> sqlx::Result<MediaAsset> {
        let row = sqlx::query_as::<_, MediaAssetRow>(
            r#"
            INSERT INTO media_assets (
                owner_id,
                kind,
                original_key,
                variants
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
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
                updated_at
            "#,
        )
        .bind(input.owner_id().as_uuid())
        .bind(input.kind().as_str())
        .bind(input.original_key())
        .bind(Json(input.variants().clone()))
        .fetch_one(pool)
        .await?;

        MediaAsset::try_from(row)
    }

    pub async fn find_asset_by_id(
        pool: &PgPool,
        id: MediaAssetId,
    ) -> sqlx::Result<Option<MediaAsset>> {
        let row = sqlx::query_as::<_, MediaAssetRow>(
            r#"
            SELECT
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
                updated_at
            FROM media_assets
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(pool)
        .await?;

        row.map(MediaAsset::try_from).transpose()
    }

    pub async fn update_asset_status(
        pool: &PgPool,
        id: MediaAssetId,
        status: MediaAssetStatus,
    ) -> sqlx::Result<MediaAsset> {
        let row = sqlx::query_as::<_, MediaAssetRow>(
            r#"
            UPDATE media_assets
            SET
                status = $2,
                updated_at = now()
            WHERE id = $1
            RETURNING
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
                updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(status.as_str())
        .fetch_one(pool)
        .await?;

        MediaAsset::try_from(row)
    }

    pub async fn enqueue_job(pool: &PgPool, input: &CreateMediaJob) -> sqlx::Result<MediaJob> {
        let row = sqlx::query_as::<_, MediaJobRow>(
            r#"
            INSERT INTO media_jobs (
                asset_id,
                kind,
                payload,
                max_attempts,
                run_after
            )
            VALUES ($1, $2, $3, $4, COALESCE($5, now()))
            RETURNING
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
                updated_at
            "#,
        )
        .bind(input.asset_id().as_uuid())
        .bind(input.kind().as_str())
        .bind(Json(input.payload().clone()))
        .bind(input.max_attempts())
        .bind(input.run_after())
        .fetch_one(pool)
        .await?;

        MediaJob::try_from(row)
    }

    pub async fn claim_next_job(pool: &PgPool, worker_id: &str) -> sqlx::Result<Option<MediaJob>> {
        let row = sqlx::query_as::<_, MediaJobRow>(
            r#"
            WITH next_job AS (
                SELECT id
                FROM media_jobs
                WHERE status = 'queued'
                    AND run_after <= now()
                    AND attempts < max_attempts
                ORDER BY run_after ASC, created_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            UPDATE media_jobs
            SET
                status = 'running',
                attempts = attempts + 1,
                locked_at = now(),
                locked_by = $1,
                updated_at = now()
            WHERE id = (SELECT id FROM next_job)
            RETURNING
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
                updated_at
            "#,
        )
        .bind(worker_id)
        .fetch_optional(pool)
        .await?;

        row.map(MediaJob::try_from).transpose()
    }

    pub async fn mark_job_succeeded(pool: &PgPool, id: MediaJobId) -> sqlx::Result<MediaJob> {
        update_job_terminal_status(pool, id, MediaJobStatus::Succeeded, None).await
    }

    pub async fn mark_job_failed(
        pool: &PgPool,
        id: MediaJobId,
        error: &str,
    ) -> sqlx::Result<MediaJob> {
        update_job_terminal_status(pool, id, MediaJobStatus::Failed, Some(error)).await
    }

    async fn update_job_terminal_status(
        pool: &PgPool,
        id: MediaJobId,
        status: MediaJobStatus,
        error: Option<&str>,
    ) -> sqlx::Result<MediaJob> {
        let row = sqlx::query_as::<_, MediaJobRow>(
            r#"
            UPDATE media_jobs
            SET
                status = $2,
                locked_at = NULL,
                locked_by = NULL,
                last_error = $3,
                updated_at = now()
            WHERE id = $1
            RETURNING
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
                updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(status.as_str())
        .bind(error)
        .fetch_one(pool)
        .await?;

        MediaJob::try_from(row)
    }

    fn decode_error(column: &'static str, message: String) -> sqlx::Error {
        sqlx::Error::ColumnDecode {
            index: column.to_owned(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        }
    }
}
