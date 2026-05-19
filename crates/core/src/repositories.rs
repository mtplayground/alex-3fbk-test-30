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
