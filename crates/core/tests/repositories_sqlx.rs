use std::env;
use std::error::Error;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use zeroclaw_core::models::{CreateMediaAsset, CreateUser, MediaAssetStatus, MediaKind, User};
use zeroclaw_core::repositories::{
    comments, follows, media, notifications, posts, social, users,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

struct TestDatabase {
    admin_pool: PgPool,
    pool: PgPool,
    schema: String,
}

impl TestDatabase {
    async fn connect() -> Result<Option<Self>, Box<dyn Error>> {
        let database_url = match env::var("TEST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
        {
            Ok(value) => value,
            Err(_) => {
                eprintln!("skipping sqlx fixture test: TEST_DATABASE_URL/DATABASE_URL is not set");
                return Ok(None);
            }
        };

        let admin_pool = match PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
        {
            Ok(pool) => pool,
            Err(error) => {
                eprintln!("skipping sqlx fixture test: database is unavailable: {error}");
                return Ok(None);
            }
        };

        let schema = format!("test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin_pool)
            .await?;

        let search_path = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |connection, _metadata| {
                let search_path = search_path.clone();
                Box::pin(async move {
                    connection
                        .execute(format!("SET search_path TO {search_path}, public").as_str())
                        .await?;
                    Ok(())
                })
            })
            .connect(&database_url)
            .await?;

        MIGRATOR.run(&pool).await?;

        Ok(Some(Self {
            admin_pool,
            pool,
            schema,
        }))
    }

    async fn cleanup(self) -> sqlx::Result<()> {
        sqlx::query(&format!("DROP SCHEMA IF EXISTS {} CASCADE", self.schema))
            .execute(&self.admin_pool)
            .await?;
        Ok(())
    }
}

async fn create_user(pool: &PgPool, label: &str) -> sqlx::Result<User> {
    let suffix = Uuid::new_v4().simple().to_string();
    let input = CreateUser::new(
        format!("{label}-{suffix}@example.com"),
        format!("{label}-{suffix}"),
        "password-hash",
        format!("{label} user"),
    );

    users::create(pool, &input).await
}

async fn create_uploaded_image(pool: &PgPool, owner: &User) -> sqlx::Result<Uuid> {
    let asset = media::create_asset(
        pool,
        &CreateMediaAsset::new(
            owner.id(),
            MediaKind::Image,
            format!("media/originals/{}.jpg", Uuid::new_v4()),
        ),
    )
    .await?;
    let asset = media::update_asset_status(pool, asset.id(), MediaAssetStatus::Uploaded).await?;

    Ok(asset.id().as_uuid())
}

async fn create_post(pool: &PgPool, author: &User, caption: &str) -> sqlx::Result<posts::Post> {
    let media_id = create_uploaded_image(pool, author).await?;
    posts::create(
        pool,
        &posts::CreatePost {
            author_id: author.id(),
            caption: caption.to_owned(),
            location: None,
            media_ids: vec![media_id],
            hashtags: Vec::new(),
            mentions: Vec::new(),
        },
    )
    .await
}

#[tokio::test]
async fn home_feed_ranks_followed_posts_by_engagement() -> Result<(), Box<dyn Error>> {
    let Some(db) = TestDatabase::connect().await? else {
        return Ok(());
    };
    let pool = &db.pool;
    let viewer = create_user(pool, "viewer").await?;
    let author = create_user(pool, "author").await?;
    let liker = create_user(pool, "liker").await?;
    let commenter = create_user(pool, "commenter").await?;
    follows::upsert(
        pool,
        viewer.id(),
        author.id(),
        follows::FollowState::Accepted,
    )
    .await?;

    let quiet_post = create_post(pool, &author, "quiet").await?;
    let engaged_post = create_post(pool, &author, "engaged").await?;
    sqlx::query("UPDATE posts SET created_at = now() - interval '1 hour' WHERE id = $1")
        .bind(quiet_post.id)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE posts SET created_at = now() - interval '1 hour' WHERE id = $1")
        .bind(engaged_post.id)
        .execute(pool)
        .await?;
    social::toggle_like(
        pool,
        liker.id(),
        social::LikeTargetKind::Post,
        engaged_post.id,
    )
    .await?;
    social::toggle_save(pool, viewer.id(), engaged_post.id).await?;
    comments::create(
        pool,
        &comments::CreateComment {
            post_id: engaged_post.id,
            parent_id: None,
            author_id: commenter.id(),
            body: "rank booster".to_owned(),
        },
    )
    .await?;

    let feed = posts::list_home_feed(pool, viewer.id(), None, 10).await?;

    assert_eq!(feed.len(), 2);
    assert_eq!(feed[0].post.id, engaged_post.id);
    assert!(feed[0].rank_score > feed[1].rank_score);

    db.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn notifications_are_listed_counted_and_marked_read() -> Result<(), Box<dyn Error>> {
    let Some(db) = TestDatabase::connect().await? else {
        return Ok(());
    };
    let pool = &db.pool;
    let recipient = create_user(pool, "recipient").await?;
    let actor = create_user(pool, "actor").await?;
    let target_id = Uuid::new_v4();

    let created = notifications::create(
        pool,
        &notifications::CreateNotification {
            user_id: recipient.id(),
            kind: notifications::NotificationKind::Follow,
            actor_id: actor.id(),
            target_kind: notifications::NotificationTargetKind::User,
            target_id,
        },
    )
    .await?;
    let self_notification = notifications::create(
        pool,
        &notifications::CreateNotification {
            user_id: actor.id(),
            kind: notifications::NotificationKind::Follow,
            actor_id: actor.id(),
            target_kind: notifications::NotificationTargetKind::User,
            target_id: actor.id().as_uuid(),
        },
    )
    .await?;

    assert!(created.is_some());
    assert!(self_notification.is_none());
    assert_eq!(notifications::unread_count(pool, recipient.id()).await?, 1);

    let listed = notifications::list_for_user(pool, recipient.id(), None, 10).await?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].kind, notifications::NotificationKind::Follow);
    assert_eq!(listed[0].target_id, target_id);
    assert!(listed[0].read_at.is_none());

    assert_eq!(notifications::mark_all_read(pool, recipient.id()).await?, 1);
    assert_eq!(notifications::unread_count(pool, recipient.id()).await?, 0);
    assert_eq!(notifications::mark_all_read(pool, recipient.id()).await?, 0);

    db.cleanup().await?;
    Ok(())
}
