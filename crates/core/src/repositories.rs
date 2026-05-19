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

    pub async fn update_asset_processing_result(
        pool: &PgPool,
        id: MediaAssetId,
        variants: Value,
        width: i32,
        height: i32,
    ) -> sqlx::Result<MediaAsset> {
        let row = sqlx::query_as::<_, MediaAssetRow>(
            r#"
            UPDATE media_assets
            SET
                status = 'ready',
                variants = $2,
                width = $3,
                height = $4,
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
        .bind(Json(variants))
        .bind(width)
        .bind(height)
        .fetch_one(pool)
        .await?;

        MediaAsset::try_from(row)
    }

    pub async fn update_asset_variants(
        pool: &PgPool,
        id: MediaAssetId,
        variants: Value,
    ) -> sqlx::Result<MediaAsset> {
        let row = sqlx::query_as::<_, MediaAssetRow>(
            r#"
            UPDATE media_assets
            SET
                status = 'ready',
                variants = $2,
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
        .bind(Json(variants))
        .fetch_one(pool)
        .await?;

        MediaAsset::try_from(row)
    }

    pub async fn enqueue_job(pool: &PgPool, input: &CreateMediaJob) -> sqlx::Result<MediaJob> {
        let row = sqlx::query_as::<_, MediaJobRow>(
            r#"
            WITH inserted AS (
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
            ),
            notified AS (
                SELECT pg_notify('media_jobs', id::text) FROM inserted
            )
            SELECT
                inserted.id,
                inserted.asset_id,
                inserted.kind,
                inserted.status,
                inserted.payload,
                inserted.attempts,
                inserted.max_attempts,
                inserted.run_after,
                inserted.locked_at,
                inserted.locked_by,
                inserted.last_error,
                inserted.created_at,
                inserted.updated_at
            FROM inserted
            CROSS JOIN notified
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

    pub async fn retry_job(
        pool: &PgPool,
        id: MediaJobId,
        run_after: DateTime<Utc>,
        error: &str,
    ) -> sqlx::Result<MediaJob> {
        let row = sqlx::query_as::<_, MediaJobRow>(
            r#"
            UPDATE media_jobs
            SET
                status = 'queued',
                run_after = $2,
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
        .bind(run_after)
        .bind(error)
        .fetch_one(pool)
        .await?;

        MediaJob::try_from(row)
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

pub mod posts {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool, Row};
    use uuid::Uuid;

    use crate::models::UserId;

    #[derive(Debug, Clone)]
    pub struct Post {
        pub id: Uuid,
        pub author_id: UserId,
        pub author_handle: String,
        pub caption: String,
        pub location: Option<String>,
        pub created_at: DateTime<Utc>,
        pub media: Vec<PostMedia>,
        pub hashtags: Vec<String>,
        pub mentions: Vec<PostMention>,
    }

    #[derive(Debug, Clone)]
    pub struct PostMedia {
        pub media_id: Uuid,
        pub position: i32,
        pub kind: String,
        pub original_key: String,
        pub variants: serde_json::Value,
        pub width: Option<i32>,
        pub height: Option<i32>,
        pub duration_ms: Option<i64>,
    }

    #[derive(Debug, Clone)]
    pub struct PostMention {
        pub user_id: UserId,
        pub handle: String,
        pub position: i32,
    }

    #[derive(Debug, Clone)]
    pub struct FeedPost {
        pub post: Post,
        pub rank_score: f64,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct FeedCursor {
        pub rank_score: f64,
        pub created_at: DateTime<Utc>,
        pub id: Uuid,
    }

    pub struct CreatePost {
        pub author_id: UserId,
        pub caption: String,
        pub location: Option<String>,
        pub media_ids: Vec<Uuid>,
        pub hashtags: Vec<String>,
        pub mentions: Vec<ParsedMention>,
    }

    pub struct ParsedMention {
        pub handle: String,
        pub position: i32,
    }

    #[derive(Debug, FromRow)]
    struct PostRow {
        id: Uuid,
        author_id: Uuid,
        author_handle: String,
        caption: String,
        location: Option<String>,
        created_at: DateTime<Utc>,
    }

    #[derive(Debug, FromRow)]
    struct FeedPostRow {
        id: Uuid,
        author_id: Uuid,
        author_handle: String,
        caption: String,
        location: Option<String>,
        created_at: DateTime<Utc>,
        rank_score: f64,
    }

    #[derive(Debug, FromRow)]
    struct PostMediaRow {
        media_id: Uuid,
        position: i32,
        kind: String,
        original_key: String,
        variants: sqlx::types::Json<serde_json::Value>,
        width: Option<i32>,
        height: Option<i32>,
        duration_ms: Option<i64>,
    }

    #[derive(Debug, FromRow)]
    struct PostMentionRow {
        user_id: Uuid,
        handle: String,
        position: i32,
    }

    impl From<PostMediaRow> for PostMedia {
        fn from(row: PostMediaRow) -> Self {
            Self {
                media_id: row.media_id,
                position: row.position,
                kind: row.kind,
                original_key: row.original_key,
                variants: row.variants.0,
                width: row.width,
                height: row.height,
                duration_ms: row.duration_ms,
            }
        }
    }

    impl From<PostMentionRow> for PostMention {
        fn from(row: PostMentionRow) -> Self {
            Self {
                user_id: UserId::from(row.user_id),
                handle: row.handle,
                position: row.position,
            }
        }
    }

    pub async fn create(pool: &PgPool, input: &CreatePost) -> sqlx::Result<Post> {
        let mut tx = pool.begin().await?;

        let usable_media_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM media_assets
            WHERE owner_id = $1
                AND id = ANY($2)
                AND status IN ('uploaded', 'processing', 'ready')
            "#,
        )
        .bind(input.author_id.as_uuid())
        .bind(&input.media_ids)
        .fetch_one(&mut *tx)
        .await?;

        if usable_media_count != input.media_ids.len() as i64 {
            return Err(sqlx::Error::RowNotFound);
        }

        let post_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO posts (author_id, caption, location)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(input.author_id.as_uuid())
        .bind(input.caption.trim())
        .bind(input.location.as_deref())
        .fetch_one(&mut *tx)
        .await?;

        for (position, media_id) in input.media_ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO post_media (post_id, media_id, position)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(post_id)
            .bind(media_id)
            .bind(position as i32)
            .execute(&mut *tx)
            .await?;
        }

        for hashtag in &input.hashtags {
            let hashtag_id: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO hashtags (name)
                VALUES ($1)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(hashtag)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO post_hashtags (post_id, hashtag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(post_id)
            .bind(hashtag_id)
            .execute(&mut *tx)
            .await?;
        }

        for mention in &input.mentions {
            if let Some(row) = sqlx::query("SELECT id, handle FROM users WHERE handle = $1")
                .bind(&mention.handle)
                .fetch_optional(&mut *tx)
                .await?
            {
                let user_id: Uuid = row.try_get("id")?;
                let handle: String = row.try_get("handle")?;
                sqlx::query(
                    r#"
                    INSERT INTO mentions (post_id, mentioned_user_id, handle, position)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(post_id)
                .bind(user_id)
                .bind(handle)
                .bind(mention.position)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        find_by_id(pool, post_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Post>> {
        let Some(row) = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT
                posts.id,
                posts.author_id,
                users.handle AS author_handle,
                posts.caption,
                posts.location,
                posts.created_at
            FROM posts
            JOIN users ON users.id = posts.author_id
            WHERE posts.id = $1
                AND posts.deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        else {
            return Ok(None);
        };

        hydrate(pool, row).await.map(Some)
    }

    pub async fn list_by_author(
        pool: &PgPool,
        author_id: UserId,
        before: Option<DateTime<Utc>>,
        limit: i64,
    ) -> sqlx::Result<Vec<Post>> {
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT
                posts.id,
                posts.author_id,
                users.handle AS author_handle,
                posts.caption,
                posts.location,
                posts.created_at
            FROM posts
            JOIN users ON users.id = posts.author_id
            WHERE posts.author_id = $1
                AND posts.deleted_at IS NULL
                AND ($2::timestamptz IS NULL OR posts.created_at < $2)
            ORDER BY posts.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(author_id.as_uuid())
        .bind(before)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut posts = Vec::with_capacity(rows.len());
        for row in rows {
            posts.push(hydrate(pool, row).await?);
        }

        Ok(posts)
    }

    pub async fn list_home_feed(
        pool: &PgPool,
        user_id: UserId,
        cursor: Option<FeedCursor>,
        limit: i64,
    ) -> sqlx::Result<Vec<FeedPost>> {
        let rows = sqlx::query_as::<_, FeedPostRow>(
            r#"
            WITH feed_posts AS (
                SELECT
                    posts.id,
                    posts.author_id,
                    users.handle AS author_handle,
                    posts.caption,
                    posts.location,
                    posts.created_at,
                    (
                        EXTRACT(EPOCH FROM posts.created_at)
                        + COUNT(DISTINCT post_likes.user_id)::double precision * 120
                        + COUNT(DISTINCT comments.id)::double precision * 90
                        + COUNT(DISTINCT saves.user_id)::double precision * 150
                    )::double precision AS rank_score
                FROM posts
                JOIN follows
                    ON follows.followee_id = posts.author_id
                    AND follows.follower_id = $1
                    AND follows.state = 'accepted'
                JOIN users ON users.id = posts.author_id
                LEFT JOIN likes AS post_likes
                    ON post_likes.target_kind = 'post'
                    AND post_likes.target_id = posts.id
                LEFT JOIN comments ON comments.post_id = posts.id
                LEFT JOIN saves ON saves.post_id = posts.id
                WHERE posts.deleted_at IS NULL
                    AND posts.created_at >= now() - interval '7 days'
                GROUP BY
                    posts.id,
                    posts.author_id,
                    users.handle,
                    posts.caption,
                    posts.location,
                    posts.created_at
            )
            SELECT
                id,
                author_id,
                author_handle,
                caption,
                location,
                created_at,
                rank_score
            FROM feed_posts
            WHERE
                $2::double precision IS NULL
                OR rank_score < $2
                OR (rank_score = $2 AND created_at < $3)
                OR (rank_score = $2 AND created_at = $3 AND id < $4)
            ORDER BY rank_score DESC, created_at DESC, id DESC
            LIMIT $5
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(cursor.map(|cursor| cursor.rank_score))
        .bind(cursor.map(|cursor| cursor.created_at))
        .bind(cursor.map(|cursor| cursor.id))
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut posts = Vec::with_capacity(rows.len());
        for row in rows {
            let rank_score = row.rank_score;
            let post_row = PostRow {
                id: row.id,
                author_id: row.author_id,
                author_handle: row.author_handle,
                caption: row.caption,
                location: row.location,
                created_at: row.created_at,
            };
            posts.push(FeedPost {
                post: hydrate(pool, post_row).await?,
                rank_score,
            });
        }

        Ok(posts)
    }

    pub async fn list_explore(
        pool: &PgPool,
        viewer_id: Option<UserId>,
        hashtag: Option<&str>,
        place: Option<&str>,
        cursor: Option<FeedCursor>,
        limit: i64,
    ) -> sqlx::Result<Vec<FeedPost>> {
        let rows = sqlx::query_as::<_, FeedPostRow>(
            r#"
            WITH explore_posts AS (
                SELECT
                    posts.id,
                    posts.author_id,
                    users.handle AS author_handle,
                    posts.caption,
                    posts.location,
                    posts.created_at,
                    (
                        COUNT(DISTINCT post_likes.user_id)::double precision * 3
                        + COUNT(DISTINCT comments.id)::double precision * 2
                        + COUNT(DISTINCT saves.user_id)::double precision * 4
                        + GREATEST(
                            0::double precision,
                            168::double precision - (EXTRACT(EPOCH FROM (now() - posts.created_at)) / 3600)
                        ) / 168::double precision
                    )::double precision AS rank_score
                FROM posts
                JOIN users ON users.id = posts.author_id
                LEFT JOIN likes AS post_likes
                    ON post_likes.target_kind = 'post'
                    AND post_likes.target_id = posts.id
                LEFT JOIN comments ON comments.post_id = posts.id
                LEFT JOIN saves ON saves.post_id = posts.id
                WHERE posts.deleted_at IS NULL
                    AND posts.created_at >= now() - interval '30 days'
                    AND ($1::uuid IS NULL OR posts.author_id <> $1)
                    AND (
                        $1::uuid IS NULL
                        OR NOT EXISTS (
                            SELECT 1
                            FROM follows
                            WHERE follows.follower_id = $1
                                AND follows.followee_id = posts.author_id
                                AND follows.state = 'accepted'
                        )
                    )
                    AND (
                        $2::text IS NULL
                        OR EXISTS (
                            SELECT 1
                            FROM post_hashtags
                            JOIN hashtags ON hashtags.id = post_hashtags.hashtag_id
                            WHERE post_hashtags.post_id = posts.id
                                AND hashtags.name = $2
                        )
                    )
                    AND (
                        $3::text IS NULL
                        OR lower(posts.location) = lower($3)
                    )
                GROUP BY
                    posts.id,
                    posts.author_id,
                    users.handle,
                    posts.caption,
                    posts.location,
                    posts.created_at
            )
            SELECT
                id,
                author_id,
                author_handle,
                caption,
                location,
                created_at,
                rank_score
            FROM explore_posts
            WHERE
                $4::double precision IS NULL
                OR rank_score < $4
                OR (rank_score = $4 AND created_at < $5)
                OR (rank_score = $4 AND created_at = $5 AND id < $6)
            ORDER BY rank_score DESC, created_at DESC, id DESC
            LIMIT $7
            "#,
        )
        .bind(viewer_id.map(|id| id.as_uuid()))
        .bind(hashtag)
        .bind(place)
        .bind(cursor.map(|cursor| cursor.rank_score))
        .bind(cursor.map(|cursor| cursor.created_at))
        .bind(cursor.map(|cursor| cursor.id))
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut posts = Vec::with_capacity(rows.len());
        for row in rows {
            let rank_score = row.rank_score;
            let post_row = PostRow {
                id: row.id,
                author_id: row.author_id,
                author_handle: row.author_handle,
                caption: row.caption,
                location: row.location,
                created_at: row.created_at,
            };
            posts.push(FeedPost {
                post: hydrate(pool, post_row).await?,
                rank_score,
            });
        }

        Ok(posts)
    }

    pub async fn soft_delete(pool: &PgPool, id: Uuid, author_id: UserId) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE posts
            SET deleted_at = now()
            WHERE id = $1
                AND author_id = $2
                AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .bind(author_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn hydrate(pool: &PgPool, row: PostRow) -> sqlx::Result<Post> {
        let media = sqlx::query_as::<_, PostMediaRow>(
            r#"
            SELECT
                media_assets.id AS media_id,
                post_media.position,
                media_assets.kind,
                media_assets.original_key,
                media_assets.variants,
                media_assets.width,
                media_assets.height,
                media_assets.duration_ms
            FROM post_media
            JOIN media_assets ON media_assets.id = post_media.media_id
            WHERE post_media.post_id = $1
            ORDER BY post_media.position ASC
            "#,
        )
        .bind(row.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(PostMedia::from)
        .collect();

        let hashtags = sqlx::query_scalar::<_, String>(
            r#"
            SELECT hashtags.name
            FROM post_hashtags
            JOIN hashtags ON hashtags.id = post_hashtags.hashtag_id
            WHERE post_hashtags.post_id = $1
            ORDER BY hashtags.name ASC
            "#,
        )
        .bind(row.id)
        .fetch_all(pool)
        .await?;

        let mentions = sqlx::query_as::<_, PostMentionRow>(
            r#"
            SELECT mentioned_user_id AS user_id, handle, position
            FROM mentions
            WHERE post_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(row.id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(PostMention::from)
        .collect();

        Ok(Post {
            id: row.id,
            author_id: UserId::from(row.author_id),
            author_handle: row.author_handle,
            caption: row.caption,
            location: row.location,
            created_at: row.created_at,
            media,
            hashtags,
            mentions,
        })
    }
}

pub mod comments {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::UserId;

    #[derive(Debug, Clone)]
    pub struct Comment {
        pub id: Uuid,
        pub post_id: Uuid,
        pub parent_id: Option<Uuid>,
        pub author_id: UserId,
        pub author_handle: String,
        pub body: String,
        pub created_at: DateTime<Utc>,
    }

    pub struct CreateComment {
        pub post_id: Uuid,
        pub parent_id: Option<Uuid>,
        pub author_id: UserId,
        pub body: String,
    }

    #[derive(Debug, FromRow)]
    struct CommentRow {
        id: Uuid,
        post_id: Uuid,
        parent_id: Option<Uuid>,
        author_id: Uuid,
        author_handle: String,
        body: String,
        created_at: DateTime<Utc>,
    }

    impl From<CommentRow> for Comment {
        fn from(row: CommentRow) -> Self {
            Self {
                id: row.id,
                post_id: row.post_id,
                parent_id: row.parent_id,
                author_id: UserId::from(row.author_id),
                author_handle: row.author_handle,
                body: row.body,
                created_at: row.created_at,
            }
        }
    }

    pub async fn create(pool: &PgPool, input: &CreateComment) -> sqlx::Result<Comment> {
        let post_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM posts
                WHERE id = $1 AND deleted_at IS NULL
            )
            "#,
        )
        .bind(input.post_id)
        .fetch_one(pool)
        .await?;

        if !post_exists {
            return Err(sqlx::Error::RowNotFound);
        }

        if let Some(parent_id) = input.parent_id {
            let parent_is_top_level: Option<bool> = sqlx::query_scalar(
                r#"
                SELECT parent_id IS NULL
                FROM comments
                WHERE id = $1 AND post_id = $2
                "#,
            )
            .bind(parent_id)
            .bind(input.post_id)
            .fetch_optional(pool)
            .await?;

            if parent_is_top_level != Some(true) {
                return Err(sqlx::Error::RowNotFound);
            }
        }

        let row = sqlx::query_as::<_, CommentRow>(
            r#"
            WITH inserted AS (
                INSERT INTO comments (post_id, parent_id, author_id, body)
                VALUES ($1, $2, $3, $4)
                RETURNING id, post_id, parent_id, author_id, body, created_at
            )
            SELECT
                inserted.id,
                inserted.post_id,
                inserted.parent_id,
                inserted.author_id,
                users.handle AS author_handle,
                inserted.body,
                inserted.created_at
            FROM inserted
            JOIN users ON users.id = inserted.author_id
            "#,
        )
        .bind(input.post_id)
        .bind(input.parent_id)
        .bind(input.author_id.as_uuid())
        .bind(input.body.trim())
        .fetch_one(pool)
        .await?;

        Ok(Comment::from(row))
    }

    pub async fn list_by_post(pool: &PgPool, post_id: Uuid) -> sqlx::Result<Vec<Comment>> {
        let post_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM posts
                WHERE id = $1 AND deleted_at IS NULL
            )
            "#,
        )
        .bind(post_id)
        .fetch_one(pool)
        .await?;

        if !post_exists {
            return Err(sqlx::Error::RowNotFound);
        }

        let rows = sqlx::query_as::<_, CommentRow>(
            r#"
            SELECT
                comments.id,
                comments.post_id,
                comments.parent_id,
                comments.author_id,
                users.handle AS author_handle,
                comments.body,
                comments.created_at
            FROM comments
            JOIN users ON users.id = comments.author_id
            WHERE comments.post_id = $1
            ORDER BY
                COALESCE(comments.parent_id, comments.id) ASC,
                comments.parent_id NULLS FIRST,
                comments.created_at ASC
            "#,
        )
        .bind(post_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Comment::from).collect())
    }

    pub async fn delete(pool: &PgPool, id: Uuid, author_id: UserId) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM comments
            WHERE id = $1 AND author_id = $2
            "#,
        )
        .bind(id)
        .bind(author_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

pub mod social {
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::models::UserId;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LikeTargetKind {
        Post,
        Comment,
    }

    impl LikeTargetKind {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Post => "post",
                Self::Comment => "comment",
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ToggleResult {
        pub active: bool,
        pub count: i64,
    }

    #[derive(Debug, Clone)]
    pub struct LikeCount {
        pub target_kind: String,
        pub target_id: Uuid,
        pub count: i64,
    }

    #[derive(Debug, Clone)]
    pub struct SaveCount {
        pub post_id: Uuid,
        pub count: i64,
    }

    pub async fn toggle_like(
        pool: &PgPool,
        user_id: UserId,
        target_kind: LikeTargetKind,
        target_id: Uuid,
    ) -> sqlx::Result<ToggleResult> {
        ensure_like_target_exists(pool, target_kind, target_id).await?;

        let inserted = sqlx::query(
            r#"
            INSERT INTO likes (user_id, target_kind, target_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(target_kind.as_str())
        .bind(target_id)
        .execute(pool)
        .await?
        .rows_affected()
            > 0;

        let active = if inserted {
            true
        } else {
            sqlx::query(
                r#"
                DELETE FROM likes
                WHERE user_id = $1 AND target_kind = $2 AND target_id = $3
                "#,
            )
            .bind(user_id.as_uuid())
            .bind(target_kind.as_str())
            .bind(target_id)
            .execute(pool)
            .await?;
            false
        };

        let count = count_likes(pool, target_kind, target_id).await?;
        Ok(ToggleResult { active, count })
    }

    pub async fn toggle_save(
        pool: &PgPool,
        user_id: UserId,
        post_id: Uuid,
    ) -> sqlx::Result<ToggleResult> {
        ensure_post_exists(pool, post_id).await?;

        let inserted = sqlx::query(
            r#"
            INSERT INTO saves (user_id, post_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id.as_uuid())
        .bind(post_id)
        .execute(pool)
        .await?
        .rows_affected()
            > 0;

        let active = if inserted {
            true
        } else {
            sqlx::query(
                r#"
                DELETE FROM saves
                WHERE user_id = $1 AND post_id = $2
                "#,
            )
            .bind(user_id.as_uuid())
            .bind(post_id)
            .execute(pool)
            .await?;
            false
        };

        let count = count_saves(pool, post_id).await?;
        Ok(ToggleResult { active, count })
    }

    pub async fn count_likes(
        pool: &PgPool,
        target_kind: LikeTargetKind,
        target_id: Uuid,
    ) -> sqlx::Result<i64> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM likes
            WHERE target_kind = $1 AND target_id = $2
            "#,
        )
        .bind(target_kind.as_str())
        .bind(target_id)
        .fetch_one(pool)
        .await
    }

    pub async fn count_saves(pool: &PgPool, post_id: Uuid) -> sqlx::Result<i64> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM saves
            WHERE post_id = $1
            "#,
        )
        .bind(post_id)
        .fetch_one(pool)
        .await
    }

    pub async fn all_like_counts(pool: &PgPool) -> sqlx::Result<Vec<LikeCount>> {
        let rows = sqlx::query_as::<_, (String, Uuid, i64)>(
            r#"
            SELECT target_kind, target_id, COUNT(*) AS count
            FROM likes
            GROUP BY target_kind, target_id
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(target_kind, target_id, count)| LikeCount {
                target_kind,
                target_id,
                count,
            })
            .collect())
    }

    pub async fn all_save_counts(pool: &PgPool) -> sqlx::Result<Vec<SaveCount>> {
        let rows = sqlx::query_as::<_, (Uuid, i64)>(
            r#"
            SELECT post_id, COUNT(*) AS count
            FROM saves
            GROUP BY post_id
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(post_id, count)| SaveCount { post_id, count })
            .collect())
    }

    async fn ensure_like_target_exists(
        pool: &PgPool,
        target_kind: LikeTargetKind,
        target_id: Uuid,
    ) -> sqlx::Result<()> {
        match target_kind {
            LikeTargetKind::Post => ensure_post_exists(pool, target_id).await,
            LikeTargetKind::Comment => {
                let exists: bool = sqlx::query_scalar(
                    r#"
                    SELECT EXISTS (
                        SELECT 1 FROM comments
                        JOIN posts ON posts.id = comments.post_id
                        WHERE comments.id = $1 AND posts.deleted_at IS NULL
                    )
                    "#,
                )
                .bind(target_id)
                .fetch_one(pool)
                .await?;

                if exists {
                    Ok(())
                } else {
                    Err(sqlx::Error::RowNotFound)
                }
            }
        }
    }

    async fn ensure_post_exists(pool: &PgPool, post_id: Uuid) -> sqlx::Result<()> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM posts
                WHERE id = $1 AND deleted_at IS NULL
            )
            "#,
        )
        .bind(post_id)
        .fetch_one(pool)
        .await?;

        if exists {
            Ok(())
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }
}

pub mod follows {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::UserId;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FollowState {
        Accepted,
        Pending,
    }

    impl FollowState {
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Accepted => "accepted",
                Self::Pending => "pending",
            }
        }

        fn from_str(value: &str) -> Option<Self> {
            match value {
                "accepted" => Some(Self::Accepted),
                "pending" => Some(Self::Pending),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Follow {
        pub follower_id: UserId,
        pub followee_id: UserId,
        pub state: FollowState,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone)]
    pub struct FollowUser {
        pub id: UserId,
        pub handle: String,
        pub display_name: String,
        pub avatar_key: Option<String>,
        pub is_private: bool,
    }

    #[derive(Debug, FromRow)]
    struct FollowRow {
        follower_id: Uuid,
        followee_id: Uuid,
        state: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    #[derive(Debug, FromRow)]
    struct FollowUserRow {
        id: Uuid,
        handle: String,
        display_name: String,
        avatar_key: Option<String>,
        is_private: bool,
    }

    impl TryFrom<FollowRow> for Follow {
        type Error = sqlx::Error;

        fn try_from(row: FollowRow) -> Result<Self, Self::Error> {
            let state =
                FollowState::from_str(&row.state).ok_or_else(|| sqlx::Error::ColumnDecode {
                    index: "state".to_owned(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unknown follow state {:?}", row.state),
                    )
                    .into(),
                })?;

            Ok(Self {
                follower_id: UserId::from(row.follower_id),
                followee_id: UserId::from(row.followee_id),
                state,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
        }
    }

    impl From<FollowUserRow> for FollowUser {
        fn from(row: FollowUserRow) -> Self {
            Self {
                id: UserId::from(row.id),
                handle: row.handle,
                display_name: row.display_name,
                avatar_key: row.avatar_key,
                is_private: row.is_private,
            }
        }
    }

    pub async fn upsert(
        pool: &PgPool,
        follower_id: UserId,
        followee_id: UserId,
        state: FollowState,
    ) -> sqlx::Result<Follow> {
        let row = sqlx::query_as::<_, FollowRow>(
            r#"
            INSERT INTO follows (follower_id, followee_id, state)
            VALUES ($1, $2, $3)
            ON CONFLICT (follower_id, followee_id)
            DO UPDATE SET state = EXCLUDED.state, updated_at = now()
            RETURNING follower_id, followee_id, state, created_at, updated_at
            "#,
        )
        .bind(follower_id.as_uuid())
        .bind(followee_id.as_uuid())
        .bind(state.as_str())
        .fetch_one(pool)
        .await?;

        Follow::try_from(row)
    }

    pub async fn delete(
        pool: &PgPool,
        follower_id: UserId,
        followee_id: UserId,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM follows
            WHERE follower_id = $1 AND followee_id = $2
            "#,
        )
        .bind(follower_id.as_uuid())
        .bind(followee_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn accept(
        pool: &PgPool,
        follower_id: UserId,
        followee_id: UserId,
    ) -> sqlx::Result<Option<Follow>> {
        let row = sqlx::query_as::<_, FollowRow>(
            r#"
            UPDATE follows
            SET state = 'accepted', updated_at = now()
            WHERE follower_id = $1
                AND followee_id = $2
                AND state = 'pending'
            RETURNING follower_id, followee_id, state, created_at, updated_at
            "#,
        )
        .bind(follower_id.as_uuid())
        .bind(followee_id.as_uuid())
        .fetch_optional(pool)
        .await?;

        row.map(Follow::try_from).transpose()
    }

    pub async fn reject(
        pool: &PgPool,
        follower_id: UserId,
        followee_id: UserId,
    ) -> sqlx::Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM follows
            WHERE follower_id = $1
                AND followee_id = $2
                AND state = 'pending'
            "#,
        )
        .bind(follower_id.as_uuid())
        .bind(followee_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_followers(
        pool: &PgPool,
        followee_id: UserId,
    ) -> sqlx::Result<Vec<FollowUser>> {
        list_users_for_follow_relation(pool, "followers", followee_id).await
    }

    pub async fn list_following(
        pool: &PgPool,
        follower_id: UserId,
    ) -> sqlx::Result<Vec<FollowUser>> {
        list_users_for_follow_relation(pool, "following", follower_id).await
    }

    async fn list_users_for_follow_relation(
        pool: &PgPool,
        relation: &str,
        user_id: UserId,
    ) -> sqlx::Result<Vec<FollowUser>> {
        let sql = match relation {
            "followers" => {
                r#"
                SELECT users.id, users.handle, users.display_name, users.avatar_key, users.is_private
                FROM follows
                JOIN users ON users.id = follows.follower_id
                WHERE follows.followee_id = $1 AND follows.state = 'accepted'
                ORDER BY follows.created_at DESC
                "#
            }
            _ => {
                r#"
                SELECT users.id, users.handle, users.display_name, users.avatar_key, users.is_private
                FROM follows
                JOIN users ON users.id = follows.followee_id
                WHERE follows.follower_id = $1 AND follows.state = 'accepted'
                ORDER BY follows.created_at DESC
                "#
            }
        };

        let rows = sqlx::query_as::<_, FollowUserRow>(sql)
            .bind(user_id.as_uuid())
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(FollowUser::from).collect())
    }
}

pub mod search {
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use super::posts::{self, Post};

    #[derive(Debug, Clone)]
    pub struct UserSearchResult {
        pub id: Uuid,
        pub handle: String,
        pub display_name: String,
        pub avatar_key: Option<String>,
        pub is_private: bool,
    }

    #[derive(Debug, Clone)]
    pub struct HashtagSearchResult {
        pub name: String,
        pub post_count: i64,
    }

    #[derive(Debug, FromRow)]
    struct UserSearchRow {
        id: Uuid,
        handle: String,
        display_name: String,
        avatar_key: Option<String>,
        is_private: bool,
    }

    #[derive(Debug, FromRow)]
    struct HashtagSearchRow {
        name: String,
        post_count: i64,
    }

    #[derive(Debug, FromRow)]
    struct PostSearchRow {
        id: Uuid,
    }

    impl From<UserSearchRow> for UserSearchResult {
        fn from(row: UserSearchRow) -> Self {
            Self {
                id: row.id,
                handle: row.handle,
                display_name: row.display_name,
                avatar_key: row.avatar_key,
                is_private: row.is_private,
            }
        }
    }

    impl From<HashtagSearchRow> for HashtagSearchResult {
        fn from(row: HashtagSearchRow) -> Self {
            Self {
                name: row.name,
                post_count: row.post_count,
            }
        }
    }

    pub async fn users(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> sqlx::Result<Vec<UserSearchResult>> {
        let rows = sqlx::query_as::<_, UserSearchRow>(
            r#"
            WITH search_query AS (
                SELECT websearch_to_tsquery('simple', $1) AS query
            )
            SELECT
                users.id,
                users.handle,
                users.display_name,
                users.avatar_key,
                users.is_private
            FROM users, search_query
            WHERE
                users.search_vector @@ search_query.query
                OR left(lower(users.handle), length(lower($1))) = lower($1)
                OR similarity(users.handle, $1) > 0.2
            ORDER BY
                CASE WHEN left(lower(users.handle), length(lower($1))) = lower($1) THEN 0 ELSE 1 END,
                ts_rank(users.search_vector, search_query.query) DESC,
                similarity(users.handle, $1) DESC,
                users.handle ASC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(UserSearchResult::from).collect())
    }

    pub async fn hashtags(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> sqlx::Result<Vec<HashtagSearchResult>> {
        let rows = sqlx::query_as::<_, HashtagSearchRow>(
            r#"
            SELECT
                hashtags.name,
                COUNT(post_hashtags.post_id)::bigint AS post_count
            FROM hashtags
            LEFT JOIN post_hashtags ON post_hashtags.hashtag_id = hashtags.id
            WHERE
                left(lower(hashtags.name), length(lower($1))) = lower($1)
                OR similarity(hashtags.name, $1) > 0.2
            GROUP BY hashtags.id, hashtags.name
            ORDER BY
                CASE WHEN left(lower(hashtags.name), length(lower($1))) = lower($1) THEN 0 ELSE 1 END,
                similarity(hashtags.name, $1) DESC,
                post_count DESC,
                hashtags.name ASC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(HashtagSearchResult::from).collect())
    }

    pub async fn posts_matching(pool: &PgPool, query: &str, limit: i64) -> sqlx::Result<Vec<Post>> {
        let rows = sqlx::query_as::<_, PostSearchRow>(
            r#"
            WITH search_query AS (
                SELECT websearch_to_tsquery('simple', $1) AS query
            )
            SELECT posts.id
            FROM posts, search_query
            WHERE posts.deleted_at IS NULL
                AND posts.search_vector @@ search_query.query
            ORDER BY
                ts_rank(posts.search_vector, search_query.query) DESC,
                posts.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let mut posts = Vec::with_capacity(rows.len());
        for row in rows {
            if let Some(post) = posts::find_by_id(pool, row.id).await? {
                posts.push(post);
            }
        }

        Ok(posts)
    }
}

pub mod stories {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::UserId;

    #[derive(Debug, Clone)]
    pub struct CreateStory {
        pub author_id: UserId,
        pub media_id: Uuid,
    }

    #[derive(Debug, Clone)]
    pub struct Story {
        pub id: Uuid,
        pub author: StoryAuthor,
        pub media: StoryMedia,
        pub created_at: DateTime<Utc>,
        pub expires_at: DateTime<Utc>,
        pub viewer_count: i64,
        pub viewed_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Clone)]
    pub struct StoryAuthor {
        pub id: UserId,
        pub handle: String,
        pub display_name: String,
        pub avatar_key: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct StoryMedia {
        pub id: Uuid,
        pub kind: String,
        pub status: String,
        pub original_key: String,
        pub variants: serde_json::Value,
        pub width: Option<i32>,
        pub height: Option<i32>,
        pub duration_ms: Option<i64>,
    }

    #[derive(Debug, Clone)]
    pub struct StoryViewer {
        pub id: UserId,
        pub handle: String,
        pub display_name: String,
        pub avatar_key: Option<String>,
        pub viewed_at: DateTime<Utc>,
    }

    #[derive(Debug, FromRow)]
    struct StoryRow {
        id: Uuid,
        author_id: Uuid,
        author_handle: String,
        author_display_name: String,
        author_avatar_key: Option<String>,
        media_id: Uuid,
        media_kind: String,
        media_status: String,
        original_key: String,
        variants: sqlx::types::Json<serde_json::Value>,
        width: Option<i32>,
        height: Option<i32>,
        duration_ms: Option<i64>,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        viewer_count: i64,
        viewed_at: Option<DateTime<Utc>>,
    }

    #[derive(Debug, FromRow)]
    struct StoryViewerRow {
        id: Uuid,
        handle: String,
        display_name: String,
        avatar_key: Option<String>,
        viewed_at: DateTime<Utc>,
    }

    impl From<StoryRow> for Story {
        fn from(row: StoryRow) -> Self {
            Self {
                id: row.id,
                author: StoryAuthor {
                    id: UserId::from(row.author_id),
                    handle: row.author_handle,
                    display_name: row.author_display_name,
                    avatar_key: row.author_avatar_key,
                },
                media: StoryMedia {
                    id: row.media_id,
                    kind: row.media_kind,
                    status: row.media_status,
                    original_key: row.original_key,
                    variants: row.variants.0,
                    width: row.width,
                    height: row.height,
                    duration_ms: row.duration_ms,
                },
                created_at: row.created_at,
                expires_at: row.expires_at,
                viewer_count: row.viewer_count,
                viewed_at: row.viewed_at,
            }
        }
    }

    impl From<StoryViewerRow> for StoryViewer {
        fn from(row: StoryViewerRow) -> Self {
            Self {
                id: UserId::from(row.id),
                handle: row.handle,
                display_name: row.display_name,
                avatar_key: row.avatar_key,
                viewed_at: row.viewed_at,
            }
        }
    }

    pub async fn create(pool: &PgPool, input: &CreateStory) -> sqlx::Result<Story> {
        let story_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO stories (author_id, media_id)
            SELECT $1, media_assets.id
            FROM media_assets
            WHERE media_assets.id = $2
                AND media_assets.owner_id = $1
                AND media_assets.status IN ('uploaded', 'processing', 'ready')
            RETURNING id
            "#,
        )
        .bind(input.author_id.as_uuid())
        .bind(input.media_id)
        .fetch_one(pool)
        .await?;

        find_by_id(pool, story_id, Some(input.author_id))
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        story_id: Uuid,
        viewer_id: Option<UserId>,
    ) -> sqlx::Result<Option<Story>> {
        let row = sqlx::query_as::<_, StoryRow>(SELECT_STORY_SQL)
            .bind(story_id)
            .bind(viewer_id.map(UserId::as_uuid))
            .fetch_optional(pool)
            .await?;

        Ok(row.map(Story::from))
    }

    pub async fn list_feed(pool: &PgPool, viewer_id: UserId) -> sqlx::Result<Vec<Story>> {
        let rows = sqlx::query_as::<_, StoryRow>(
            r#"
            SELECT
                stories.id,
                stories.author_id,
                users.handle AS author_handle,
                users.display_name AS author_display_name,
                users.avatar_key AS author_avatar_key,
                media_assets.id AS media_id,
                media_assets.kind AS media_kind,
                media_assets.status AS media_status,
                media_assets.original_key,
                media_assets.variants,
                media_assets.width,
                media_assets.height,
                media_assets.duration_ms,
                stories.created_at,
                stories.expires_at,
                COUNT(all_views.viewer_id)::bigint AS viewer_count,
                viewer_view.viewed_at
            FROM stories
            JOIN follows
                ON follows.followee_id = stories.author_id
                AND follows.follower_id = $1
                AND follows.state = 'accepted'
            JOIN users ON users.id = stories.author_id
            JOIN media_assets ON media_assets.id = stories.media_id
            LEFT JOIN story_views AS all_views ON all_views.story_id = stories.id
            LEFT JOIN story_views AS viewer_view
                ON viewer_view.story_id = stories.id
                AND viewer_view.viewer_id = $1
            WHERE stories.expires_at > now()
            GROUP BY
                stories.id,
                stories.author_id,
                users.handle,
                users.display_name,
                users.avatar_key,
                media_assets.id,
                media_assets.kind,
                media_assets.status,
                media_assets.original_key,
                media_assets.variants,
                media_assets.width,
                media_assets.height,
                media_assets.duration_ms,
                stories.created_at,
                stories.expires_at,
                viewer_view.viewed_at
            ORDER BY users.handle ASC, stories.created_at DESC, stories.id DESC
            "#,
        )
        .bind(viewer_id.as_uuid())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Story::from).collect())
    }

    pub async fn mark_viewed(
        pool: &PgPool,
        story_id: Uuid,
        viewer_id: UserId,
    ) -> sqlx::Result<DateTime<Utc>> {
        sqlx::query_scalar(
            r#"
            INSERT INTO story_views (story_id, viewer_id)
            SELECT stories.id, $2
            FROM stories
            WHERE stories.id = $1
                AND stories.expires_at > now()
            ON CONFLICT (story_id, viewer_id)
            DO UPDATE SET viewed_at = story_views.viewed_at
            RETURNING viewed_at
            "#,
        )
        .bind(story_id)
        .bind(viewer_id.as_uuid())
        .fetch_one(pool)
        .await
    }

    pub async fn list_viewers(
        pool: &PgPool,
        story_id: Uuid,
        author_id: UserId,
    ) -> sqlx::Result<Vec<StoryViewer>> {
        let authorized: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM stories
                WHERE id = $1 AND author_id = $2
            )
            "#,
        )
        .bind(story_id)
        .bind(author_id.as_uuid())
        .fetch_one(pool)
        .await?;

        if !authorized {
            return Err(sqlx::Error::RowNotFound);
        }

        let rows = sqlx::query_as::<_, StoryViewerRow>(
            r#"
            SELECT
                users.id,
                users.handle,
                users.display_name,
                users.avatar_key,
                story_views.viewed_at
            FROM story_views
            JOIN users ON users.id = story_views.viewer_id
            WHERE story_views.story_id = $1
            ORDER BY story_views.viewed_at DESC, users.handle ASC
            "#,
        )
        .bind(story_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(StoryViewer::from).collect())
    }

    pub async fn delete_expired_before(
        pool: &PgPool,
        cutoff: DateTime<Utc>,
    ) -> sqlx::Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM stories
            WHERE expires_at < $1
            "#,
        )
        .bind(cutoff)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    const SELECT_STORY_SQL: &str = r#"
        SELECT
            stories.id,
            stories.author_id,
            users.handle AS author_handle,
            users.display_name AS author_display_name,
            users.avatar_key AS author_avatar_key,
            media_assets.id AS media_id,
            media_assets.kind AS media_kind,
            media_assets.status AS media_status,
            media_assets.original_key,
            media_assets.variants,
            media_assets.width,
            media_assets.height,
            media_assets.duration_ms,
            stories.created_at,
            stories.expires_at,
            COUNT(all_views.viewer_id)::bigint AS viewer_count,
            viewer_view.viewed_at
        FROM stories
        JOIN users ON users.id = stories.author_id
        JOIN media_assets ON media_assets.id = stories.media_id
        LEFT JOIN story_views AS all_views ON all_views.story_id = stories.id
        LEFT JOIN story_views AS viewer_view
            ON viewer_view.story_id = stories.id
            AND viewer_view.viewer_id = $2
        WHERE stories.id = $1
        GROUP BY
            stories.id,
            stories.author_id,
            users.handle,
            users.display_name,
            users.avatar_key,
            media_assets.id,
            media_assets.kind,
            media_assets.status,
            media_assets.original_key,
            media_assets.variants,
            media_assets.width,
            media_assets.height,
            media_assets.duration_ms,
            stories.created_at,
            stories.expires_at,
            viewer_view.viewed_at
    "#;
}

pub mod reels {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::UserId;

    #[derive(Debug, Clone)]
    pub struct Reel {
        pub id: Uuid,
        pub author_id: UserId,
        pub author_handle: String,
        pub caption: String,
        pub media: ReelMedia,
        pub duration_ms: Option<i64>,
        pub audio_title: Option<String>,
        pub audio_artist: Option<String>,
        pub audio_is_original: bool,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone)]
    pub struct ReelMedia {
        pub media_id: Uuid,
        pub kind: String,
        pub status: String,
        pub original_key: String,
        pub variants: serde_json::Value,
        pub width: Option<i32>,
        pub height: Option<i32>,
        pub duration_ms: Option<i64>,
    }

    #[derive(Debug, Clone)]
    pub struct FeedReel {
        pub reel: Reel,
        pub rank_score: f64,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct FeedCursor {
        pub rank_score: f64,
        pub created_at: DateTime<Utc>,
        pub id: Uuid,
    }

    pub struct CreateReel {
        pub author_id: UserId,
        pub media_id: Uuid,
        pub caption: String,
        pub duration_ms: Option<i64>,
        pub audio_title: Option<String>,
        pub audio_artist: Option<String>,
        pub audio_is_original: bool,
    }

    #[derive(Debug, FromRow)]
    struct ReelRow {
        id: Uuid,
        author_id: Uuid,
        author_handle: String,
        caption: String,
        media_id: Uuid,
        media_kind: String,
        media_status: String,
        original_key: String,
        variants: sqlx::types::Json<serde_json::Value>,
        width: Option<i32>,
        height: Option<i32>,
        media_duration_ms: Option<i64>,
        duration_ms: Option<i64>,
        audio_title: Option<String>,
        audio_artist: Option<String>,
        audio_is_original: bool,
        created_at: DateTime<Utc>,
    }

    #[derive(Debug, FromRow)]
    struct FeedReelRow {
        id: Uuid,
        author_id: Uuid,
        author_handle: String,
        caption: String,
        media_id: Uuid,
        media_kind: String,
        media_status: String,
        original_key: String,
        variants: sqlx::types::Json<serde_json::Value>,
        width: Option<i32>,
        height: Option<i32>,
        media_duration_ms: Option<i64>,
        duration_ms: Option<i64>,
        audio_title: Option<String>,
        audio_artist: Option<String>,
        audio_is_original: bool,
        created_at: DateTime<Utc>,
        rank_score: f64,
    }

    impl From<ReelRow> for Reel {
        fn from(row: ReelRow) -> Self {
            Self {
                id: row.id,
                author_id: UserId::from(row.author_id),
                author_handle: row.author_handle,
                caption: row.caption,
                media: ReelMedia {
                    media_id: row.media_id,
                    kind: row.media_kind,
                    status: row.media_status,
                    original_key: row.original_key,
                    variants: row.variants.0,
                    width: row.width,
                    height: row.height,
                    duration_ms: row.media_duration_ms,
                },
                duration_ms: row.duration_ms,
                audio_title: row.audio_title,
                audio_artist: row.audio_artist,
                audio_is_original: row.audio_is_original,
                created_at: row.created_at,
            }
        }
    }

    impl From<FeedReelRow> for FeedReel {
        fn from(row: FeedReelRow) -> Self {
            let rank_score = row.rank_score;
            let reel = Reel::from(ReelRow {
                id: row.id,
                author_id: row.author_id,
                author_handle: row.author_handle,
                caption: row.caption,
                media_id: row.media_id,
                media_kind: row.media_kind,
                media_status: row.media_status,
                original_key: row.original_key,
                variants: row.variants,
                width: row.width,
                height: row.height,
                media_duration_ms: row.media_duration_ms,
                duration_ms: row.duration_ms,
                audio_title: row.audio_title,
                audio_artist: row.audio_artist,
                audio_is_original: row.audio_is_original,
                created_at: row.created_at,
            });

            Self { reel, rank_score }
        }
    }

    pub async fn create(pool: &PgPool, input: &CreateReel) -> sqlx::Result<Reel> {
        let reel_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO reels (
                author_id,
                media_id,
                caption,
                duration_ms,
                audio_title,
                audio_artist,
                audio_is_original
            )
            SELECT
                $1,
                media_assets.id,
                $3,
                COALESCE($4, media_assets.duration_ms),
                $5,
                $6,
                $7
            FROM media_assets
            WHERE media_assets.id = $2
                AND media_assets.owner_id = $1
                AND media_assets.kind = 'video'
                AND media_assets.status IN ('uploaded', 'processing', 'ready')
            RETURNING id
            "#,
        )
        .bind(input.author_id.as_uuid())
        .bind(input.media_id)
        .bind(input.caption.trim())
        .bind(input.duration_ms)
        .bind(input.audio_title.as_deref())
        .bind(input.audio_artist.as_deref())
        .bind(input.audio_is_original)
        .fetch_one(pool)
        .await?;

        find_by_id(pool, reel_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Reel>> {
        let row = sqlx::query_as::<_, ReelRow>(SELECT_REEL_SQL)
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(Reel::from))
    }

    pub async fn list_feed(
        pool: &PgPool,
        viewer_id: UserId,
        cursor: Option<FeedCursor>,
        limit: i64,
    ) -> sqlx::Result<Vec<FeedReel>> {
        let rows = sqlx::query_as::<_, FeedReelRow>(
            r#"
            WITH ranked_reels AS (
                SELECT
                    reels.id,
                    reels.author_id,
                    users.handle AS author_handle,
                    reels.caption,
                    media_assets.id AS media_id,
                    media_assets.kind AS media_kind,
                    media_assets.status AS media_status,
                    media_assets.original_key,
                    media_assets.variants,
                    media_assets.width,
                    media_assets.height,
                    media_assets.duration_ms AS media_duration_ms,
                    reels.duration_ms,
                    reels.audio_title,
                    reels.audio_artist,
                    reels.audio_is_original,
                    reels.created_at,
                    (
                        EXTRACT(EPOCH FROM reels.created_at)
                        + CASE WHEN follows.follower_id IS NULL THEN 0 ELSE 3600 END
                    )::double precision AS rank_score
                FROM reels
                JOIN users ON users.id = reels.author_id
                JOIN media_assets ON media_assets.id = reels.media_id
                LEFT JOIN follows
                    ON follows.follower_id = $1
                    AND follows.followee_id = reels.author_id
                    AND follows.state = 'accepted'
                WHERE reels.deleted_at IS NULL
                    AND media_assets.status IN ('uploaded', 'processing', 'ready')
                    AND reels.created_at >= now() - interval '30 days'
            )
            SELECT
                id,
                author_id,
                author_handle,
                caption,
                media_id,
                media_kind,
                media_status,
                original_key,
                variants,
                width,
                height,
                media_duration_ms,
                duration_ms,
                audio_title,
                audio_artist,
                audio_is_original,
                created_at,
                rank_score
            FROM ranked_reels
            WHERE
                $2::double precision IS NULL
                OR rank_score < $2
                OR (rank_score = $2 AND created_at < $3)
                OR (rank_score = $2 AND created_at = $3 AND id < $4)
            ORDER BY rank_score DESC, created_at DESC, id DESC
            LIMIT $5
            "#,
        )
        .bind(viewer_id.as_uuid())
        .bind(cursor.map(|cursor| cursor.rank_score))
        .bind(cursor.map(|cursor| cursor.created_at))
        .bind(cursor.map(|cursor| cursor.id))
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(FeedReel::from).collect())
    }

    const SELECT_REEL_SQL: &str = r#"
        SELECT
            reels.id,
            reels.author_id,
            users.handle AS author_handle,
            reels.caption,
            media_assets.id AS media_id,
            media_assets.kind AS media_kind,
            media_assets.status AS media_status,
            media_assets.original_key,
            media_assets.variants,
            media_assets.width,
            media_assets.height,
            media_assets.duration_ms AS media_duration_ms,
            reels.duration_ms,
            reels.audio_title,
            reels.audio_artist,
            reels.audio_is_original,
            reels.created_at
        FROM reels
        JOIN users ON users.id = reels.author_id
        JOIN media_assets ON media_assets.id = reels.media_id
        WHERE reels.id = $1
            AND reels.deleted_at IS NULL
    "#;
}
