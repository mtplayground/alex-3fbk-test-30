use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::abuse::rate_limit_write_requests;
use crate::admin::{list_pending_reports, take_report_action};
use crate::auth::{forgot_password, login, logout, refresh, reset_password, signup, verify_email};
use crate::comments::{create_comment, delete_comment, get_post_comments};
use crate::conversations::{
    create_conversation, create_message, list_conversations, list_messages, mark_conversation_read,
};
use crate::follows::{
    accept_follow_request, follow_user, get_followers, get_following, reject_follow_request,
    unfollow_user,
};
use crate::health::healthz;
use crate::media::{complete_upload, create_upload};
use crate::moderation::{block_user, create_report, unblock_user};
use crate::notifications::{
    list_notifications, mark_all_notifications_read, unread_notification_count,
};
use crate::posts::{create_post, delete_post, get_explore, get_feed, get_post, get_user_posts};
use crate::profile::{create_avatar_upload, get_user_profile, update_me};
use crate::reels::{create_reel, get_reel, get_reels_feed};
use crate::search::search;
use crate::social::{toggle_comment_like, toggle_post_like, toggle_post_save};
use crate::state::AppState;
use crate::stories::{create_story, get_stories_feed, get_story_viewers, view_story};
use crate::ws::websocket_handler;

const MAX_REQUEST_BODY_BYTES: usize = 512 * 1024 * 1024;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(websocket_handler))
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/media/uploads", post(create_upload))
        .route("/media/uploads/:id/complete", post(complete_upload))
        .route("/notifications", get(list_notifications))
        .route("/notifications/read-all", post(mark_all_notifications_read))
        .route("/notifications/unread-count", get(unread_notification_count))
        .route("/admin/reports", get(list_pending_reports))
        .route("/admin/reports/:id/actions", post(take_report_action))
        .route("/reports", post(create_report))
        .route("/explore", get(get_explore))
        .route("/feed", get(get_feed))
        .route("/search", get(search))
        .route(
            "/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/conversations/:id/messages",
            get(list_messages).post(create_message),
        )
        .route("/conversations/:id/read", post(mark_conversation_read))
        .route("/stories", post(create_story))
        .route("/stories/feed", get(get_stories_feed))
        .route("/stories/:id/view", post(view_story))
        .route("/stories/:id/viewers", get(get_story_viewers))
        .route("/reels", post(create_reel))
        .route("/reels/feed", get(get_reels_feed))
        .route("/reels/:id", get(get_reel))
        .route("/posts", post(create_post))
        .route("/posts/:id", get(get_post).delete(delete_post))
        .route("/posts/:id/like", post(toggle_post_like))
        .route("/posts/:id/save", post(toggle_post_save))
        .route(
            "/posts/:id/comments",
            get(get_post_comments).post(create_comment),
        )
        .route("/comments/:id", axum::routing::delete(delete_comment))
        .route("/comments/:id/like", post(toggle_comment_like))
        .route("/users/:handle", get(get_user_profile))
        .route("/users/:handle/posts", get(get_user_posts))
        .route(
            "/users/:handle/follow",
            post(follow_user).delete(unfollow_user),
        )
        .route(
            "/users/:handle/block",
            post(block_user).delete(unblock_user),
        )
        .route("/users/:handle/followers", get(get_followers))
        .route("/users/:handle/following", get(get_following))
        .route(
            "/follow-requests/:follower_id/accept",
            post(accept_follow_request),
        )
        .route(
            "/follow-requests/:follower_id/reject",
            post(reject_follow_request),
        )
        .route("/me", axum::routing::patch(update_me))
        .route("/me/avatar", post(create_avatar_upload))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_write_requests,
        ))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
