use std::env;
use std::error::Error;

use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;
use zeroclaw_core::auth::hash_password;
use zeroclaw_core::db::run_migrations;
use zeroclaw_core::models::{
    ConversationKind, CreateConversation, CreateMediaAsset, CreateMessage, CreateUser,
    MediaAssetId, MediaAssetStatus, MediaKind, User, UserId,
};
use zeroclaw_core::repositories::{
    comments, conversations, follows, media, notifications, posts, social, stories, users,
};

type SeedResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const DEMO_PASSWORD: &str = "password123";
const SENTINEL_CAPTION: &str =
    "Welcome to Yet another Instagram seed data #yetanotherinstagram #welcome with @bob and @mira";

#[tokio::main]
async fn main() -> SeedResult<()> {
    let confirmed = env::var("ZEROCLAW_SEED_CONFIRM").ok().as_deref() == Some("1")
        || env::args().any(|arg| arg == "--yes" || arg == "-y");

    if !confirmed {
        return Err(
            "refusing to seed without ZEROCLAW_SEED_CONFIRM=1 or --yes; use local/staging only"
                .into(),
        );
    }

    let database_url =
        env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set before running seed")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    run_migrations(&pool).await?;
    seed(&pool).await?;

    println!("seed complete");
    println!("demo password for all seeded users: {DEMO_PASSWORD}");
    Ok(())
}

async fn seed(pool: &PgPool) -> SeedResult<()> {
    let alice = ensure_user(
        pool,
        UserSeed {
            email: "alice@example.test",
            handle: "alice",
            display_name: "Alice Park",
            bio: "Photographer sharing field notes from the city.",
            link: Some("https://example.test/alice"),
            avatar_key: Some("seed/avatars/alice.jpg"),
            is_private: false,
            is_admin: false,
        },
    )
    .await?;
    let bob = ensure_user(
        pool,
        UserSeed {
            email: "bob@example.test",
            handle: "bob",
            display_name: "Bob Stone",
            bio: "Coffee, architecture, and small details.",
            link: Some("https://example.test/bob"),
            avatar_key: Some("seed/avatars/bob.jpg"),
            is_private: false,
            is_admin: false,
        },
    )
    .await?;
    let mira = ensure_user(
        pool,
        UserSeed {
            email: "mira@example.test",
            handle: "mira",
            display_name: "Mira Chen",
            bio: "Private account for close friends and travel drafts.",
            link: None,
            avatar_key: Some("seed/avatars/mira.jpg"),
            is_private: true,
            is_admin: false,
        },
    )
    .await?;
    let admin = ensure_user(
        pool,
        UserSeed {
            email: "admin@example.test",
            handle: "admin",
            display_name: "Yet another Instagram Admin",
            bio: "Demo moderation account.",
            link: None,
            avatar_key: Some("seed/avatars/admin.jpg"),
            is_private: false,
            is_admin: true,
        },
    )
    .await?;

    if seed_content_exists(pool).await? {
        println!("seed content already exists; users verified and left unchanged");
        return Ok(());
    }

    follows::upsert(pool, bob.id(), alice.id(), follows::FollowState::Accepted).await?;
    follows::upsert(pool, mira.id(), alice.id(), follows::FollowState::Accepted).await?;
    follows::upsert(pool, alice.id(), bob.id(), follows::FollowState::Accepted).await?;
    follows::upsert(pool, alice.id(), mira.id(), follows::FollowState::Pending).await?;

    let alice_photo = ensure_image_asset(
        pool,
        alice.id(),
        "seed/posts/alice-welcome-original.jpg",
        "seed/posts/alice-welcome",
        1440,
        1080,
    )
    .await?;
    let bob_photo = ensure_image_asset(
        pool,
        bob.id(),
        "seed/posts/bob-cafe-original.jpg",
        "seed/posts/bob-cafe",
        1080,
        1350,
    )
    .await?;
    let story_media = ensure_image_asset(
        pool,
        alice.id(),
        "seed/stories/alice-morning-original.jpg",
        "seed/stories/alice-morning",
        1080,
        1920,
    )
    .await?;

    let welcome_post = posts::create(
        pool,
        &posts::CreatePost {
            author_id: alice.id(),
            caption: SENTINEL_CAPTION.to_owned(),
            location: Some("San Francisco, CA".to_owned()),
            media_ids: vec![alice_photo.as_uuid()],
            hashtags: vec!["yetanotherinstagram".to_owned(), "welcome".to_owned()],
            mentions: vec![
                posts::ParsedMention {
                    handle: "bob".to_owned(),
                    position: 48,
                },
                posts::ParsedMention {
                    handle: "mira".to_owned(),
                    position: 57,
                },
            ],
        },
    )
    .await?;

    let cafe_post = posts::create(
        pool,
        &posts::CreatePost {
            author_id: bob.id(),
            caption: "Morning cafe light for the local feed #coffee".to_owned(),
            location: Some("Oakland, CA".to_owned()),
            media_ids: vec![bob_photo.as_uuid()],
            hashtags: vec!["coffee".to_owned()],
            mentions: Vec::new(),
        },
    )
    .await?;

    let comment = comments::create(
        pool,
        &comments::CreateComment {
            post_id: welcome_post.id,
            parent_id: None,
            author_id: bob.id(),
            body: "This makes the staging feed feel alive.".to_owned(),
        },
    )
    .await?;
    comments::create(
        pool,
        &comments::CreateComment {
            post_id: welcome_post.id,
            parent_id: Some(comment.id),
            author_id: alice.id(),
            body: "Exactly the goal.".to_owned(),
        },
    )
    .await?;

    social::toggle_like(pool, bob.id(), social::LikeTargetKind::Post, welcome_post.id).await?;
    social::toggle_like(pool, mira.id(), social::LikeTargetKind::Post, welcome_post.id).await?;
    social::toggle_like(pool, alice.id(), social::LikeTargetKind::Post, cafe_post.id).await?;
    social::toggle_save(pool, alice.id(), cafe_post.id).await?;

    let story = stories::create(
        pool,
        &stories::CreateStory {
            author_id: alice.id(),
            media_id: story_media.as_uuid(),
        },
    )
    .await?;
    stories::mark_viewed(pool, story.id, bob.id()).await?;

    let conversation = conversations::create(
        pool,
        &CreateConversation::new(ConversationKind::Dm).with_title("Alice and Bob"),
    )
    .await?;
    conversations::add_member(pool, conversation.id(), alice.id()).await?;
    conversations::add_member(pool, conversation.id(), bob.id()).await?;
    let dm = conversations::create_message(
        pool,
        &CreateMessage::new(
            conversation.id(),
            alice.id(),
            "Seeded hello from Alice. Realtime clients should see future messages here.",
        ),
    )
    .await?;
    conversations::create_message(
        pool,
        &CreateMessage::new(conversation.id(), bob.id(), "Received. The thread is ready."),
    )
    .await?;

    notifications::create(
        pool,
        &notifications::CreateNotification {
            user_id: alice.id(),
            kind: notifications::NotificationKind::Like,
            actor_id: bob.id(),
            target_kind: notifications::NotificationTargetKind::Post,
            target_id: welcome_post.id,
        },
    )
    .await?;
    notifications::create(
        pool,
        &notifications::CreateNotification {
            user_id: alice.id(),
            kind: notifications::NotificationKind::Comment,
            actor_id: bob.id(),
            target_kind: notifications::NotificationTargetKind::Comment,
            target_id: comment.id,
        },
    )
    .await?;
    notifications::create(
        pool,
        &notifications::CreateNotification {
            user_id: alice.id(),
            kind: notifications::NotificationKind::Follow,
            actor_id: mira.id(),
            target_kind: notifications::NotificationTargetKind::User,
            target_id: mira.id().as_uuid(),
        },
    )
    .await?;
    notifications::create(
        pool,
        &notifications::CreateNotification {
            user_id: bob.id(),
            kind: notifications::NotificationKind::Dm,
            actor_id: alice.id(),
            target_kind: notifications::NotificationTargetKind::Message,
            target_id: dm.id().as_uuid(),
        },
    )
    .await?;

    let _ = admin;
    println!("created demo posts: {}, {}", welcome_post.id, cafe_post.id);
    Ok(())
}

struct UserSeed {
    email: &'static str,
    handle: &'static str,
    display_name: &'static str,
    bio: &'static str,
    link: Option<&'static str>,
    avatar_key: Option<&'static str>,
    is_private: bool,
    is_admin: bool,
}

async fn ensure_user(pool: &PgPool, seed: UserSeed) -> SeedResult<User> {
    let existing = users::find_by_handle(pool, seed.handle).await?;
    let user = if let Some(user) = existing {
        user
    } else {
        let password_hash = hash_password(DEMO_PASSWORD)?;
        let mut input = CreateUser::new(seed.email, seed.handle, password_hash, seed.display_name)
            .with_bio(seed.bio)
            .private(seed.is_private);

        if let Some(link) = seed.link {
            input = input.with_link(link);
        }
        if let Some(avatar_key) = seed.avatar_key {
            input = input.with_avatar_key(avatar_key);
        }

        users::create(pool, &input).await?
    };

    let verified_user = users::mark_email_verified(pool, user.id()).await?;
    let final_user = if seed.is_admin {
        sqlx::query("UPDATE users SET is_admin = true WHERE id = $1")
            .bind(verified_user.id().as_uuid())
            .execute(pool)
            .await?;
        users::find_by_id(pool, verified_user.id())
            .await?
            .ok_or("seeded admin user disappeared")?
    } else {
        verified_user
    };

    println!(
        "user ready: {} ({})",
        final_user.handle(),
        final_user.email()
    );
    Ok(final_user)
}

async fn seed_content_exists(pool: &PgPool) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM posts
            WHERE caption = $1 AND deleted_at IS NULL
        )
        "#,
    )
    .bind(SENTINEL_CAPTION)
    .fetch_one(pool)
    .await
}

async fn ensure_image_asset(
    pool: &PgPool,
    owner_id: UserId,
    original_key: &str,
    prefix: &str,
    width: i32,
    height: i32,
) -> SeedResult<MediaAssetId> {
    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM media_assets WHERE original_key = $1 AND owner_id = $2",
    )
    .bind(original_key)
    .bind(owner_id.as_uuid())
    .fetch_optional(pool)
    .await?
    {
        return Ok(MediaAssetId::from(id));
    }

    let variants = json!({
        "thumb": {
            "key": format!("{prefix}-thumb.jpg"),
            "width": 240,
            "height": scaled_height(width, height, 240),
            "content_type": "image/jpeg"
        },
        "medium": {
            "key": format!("{prefix}-medium.jpg"),
            "width": 720,
            "height": scaled_height(width, height, 720),
            "content_type": "image/jpeg"
        },
        "large": {
            "key": format!("{prefix}-large.jpg"),
            "width": 1080,
            "height": scaled_height(width, height, 1080),
            "content_type": "image/jpeg"
        }
    });

    let asset = media::create_asset(
        pool,
        &CreateMediaAsset::new(owner_id, MediaKind::Image, original_key),
    )
    .await?;
    media::update_asset_status(pool, asset.id(), MediaAssetStatus::Uploaded).await?;
    let ready_asset =
        media::update_asset_processing_result(pool, asset.id(), variants, width, height).await?;

    Ok(ready_asset.id())
}

fn scaled_height(width: i32, height: i32, target_width: i32) -> i32 {
    let scaled = (i64::from(height) * i64::from(target_width)) / i64::from(width.max(1));
    scaled.max(1).min(i64::from(i32::MAX)) as i32
}
