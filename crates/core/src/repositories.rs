pub mod users {
    use chrono::{DateTime, Utc};
    use sqlx::{FromRow, PgPool};
    use uuid::Uuid;

    use crate::models::{CreateUser, User, UserId};

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
