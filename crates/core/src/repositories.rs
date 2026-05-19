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
