use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use zeroclaw_core::models::{
    CreateMediaAsset, CreateMediaJob, MediaAsset, MediaAssetId, MediaAssetStatus, MediaJobKind,
    MediaKind,
};
use zeroclaw_core::repositories::media;

use crate::error::AppError;
use crate::extractors::AuthUser;
use crate::state::AppState;

const MEDIA_UPLOAD_EXPIRES_SECONDS: u64 = 15 * 60;
const MAX_IMAGE_UPLOAD_BYTES: u64 = 15 * 1024 * 1024;
const MAX_VIDEO_UPLOAD_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct CreateUploadRequest {
    kind: MediaKind,
    content_type: Option<String>,
    size_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateUploadResponse {
    asset_id: String,
    key: String,
    upload_url: String,
    method: &'static str,
    expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct CompleteUploadResponse {
    asset_id: String,
    status: MediaAssetStatus,
    job_id: String,
    job_kind: MediaJobKind,
}

pub async fn create_upload(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateUploadRequest>,
) -> Result<(StatusCode, Json<CreateUploadResponse>), AppError> {
    let content_type = normalize_content_type(payload.content_type);
    validate_content_type(payload.kind, content_type.as_deref())?;
    validate_upload_size(payload.kind, payload.size_bytes)?;

    let extension = media_extension(payload.kind, content_type.as_deref());
    let key = format!(
        "media/originals/{}/{}/{}.{}",
        auth_user.id(),
        payload.kind.as_str(),
        Uuid::new_v4(),
        extension
    );
    let input = CreateMediaAsset::new(auth_user.id(), payload.kind, key.clone());
    let asset = media::create_asset(state.db_pool(), &input).await?;
    let expires_in = Duration::from_secs(MEDIA_UPLOAD_EXPIRES_SECONDS);
    let presigned = state
        .storage()
        .presigned_put(&key, expires_in, content_type.as_deref())
        .await?;
    let response = CreateUploadResponse {
        asset_id: asset.id().to_string(),
        key,
        upload_url: presigned.url().to_owned(),
        method: presigned.method(),
        expires_in: presigned.expires_in().as_secs(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn complete_upload(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CompleteUploadResponse>), AppError> {
    let asset_id = MediaAssetId::from(id);
    let Some(asset) = media::find_asset_by_id(state.db_pool(), asset_id).await? else {
        return Err(AppError::NotFound);
    };

    if asset.owner_id() != auth_user.id() {
        return Err(AppError::NotFound);
    }

    if asset.status() != MediaAssetStatus::Pending {
        return Err(AppError::Conflict("media upload is already complete"));
    }

    let updated_asset =
        media::update_asset_status(state.db_pool(), asset.id(), MediaAssetStatus::Uploaded).await?;
    let job_kind = job_kind_for_asset(&updated_asset);
    let job_input = CreateMediaJob::new(updated_asset.id(), job_kind).with_payload(json!({
        "asset_id": updated_asset.id().to_string(),
        "owner_id": updated_asset.owner_id().to_string(),
        "kind": updated_asset.kind().as_str(),
        "original_key": updated_asset.original_key(),
    }));
    let job = media::enqueue_job(state.db_pool(), &job_input).await?;
    let response = CompleteUploadResponse {
        asset_id: updated_asset.id().to_string(),
        status: updated_asset.status(),
        job_id: job.id().to_string(),
        job_kind: job.kind(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

fn normalize_content_type(value: Option<String>) -> Option<String> {
    value
        .map(|content_type| content_type.trim().to_ascii_lowercase())
        .filter(|content_type| !content_type.is_empty())
}

fn validate_content_type(kind: MediaKind, content_type: Option<&str>) -> Result<(), AppError> {
    let content_type = content_type.ok_or(AppError::BadRequest("content_type"))?;

    let allowed = match kind {
        MediaKind::Image => matches!(
            content_type,
            "image/jpeg" | "image/png" | "image/webp" | "image/gif"
        ),
        MediaKind::Video => matches!(
            content_type,
            "video/mp4" | "video/webm" | "video/quicktime" | "video/mpeg"
        ),
    };

    if !allowed {
        return Err(AppError::BadRequest("content_type"));
    }

    Ok(())
}

fn validate_upload_size(kind: MediaKind, size_bytes: Option<u64>) -> Result<(), AppError> {
    let Some(size_bytes) = size_bytes else {
        return Ok(());
    };
    if size_bytes == 0 {
        return Err(AppError::BadRequest("size_bytes"));
    }

    let max_size = match kind {
        MediaKind::Image => MAX_IMAGE_UPLOAD_BYTES,
        MediaKind::Video => MAX_VIDEO_UPLOAD_BYTES,
    };
    if size_bytes > max_size {
        return Err(AppError::BadRequest("size_bytes"));
    }

    Ok(())
}

fn media_extension(kind: MediaKind, content_type: Option<&str>) -> &'static str {
    match (kind, content_type) {
        (MediaKind::Image, Some("image/png")) => "png",
        (MediaKind::Image, Some("image/webp")) => "webp",
        (MediaKind::Image, Some("image/gif")) => "gif",
        (MediaKind::Image, _) => "jpg",
        (MediaKind::Video, Some("video/webm")) => "webm",
        (MediaKind::Video, Some("video/quicktime")) => "mov",
        (MediaKind::Video, Some("video/mpeg")) => "mpeg",
        (MediaKind::Video, _) => "mp4",
    }
}

fn job_kind_for_asset(asset: &MediaAsset) -> MediaJobKind {
    match asset.kind() {
        MediaKind::Image => MediaJobKind::ImageProcessing,
        MediaKind::Video => MediaJobKind::VideoProcessing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_extension_uses_kind_and_content_type() {
        assert_eq!(media_extension(MediaKind::Image, Some("image/png")), "png");
        assert_eq!(media_extension(MediaKind::Image, Some("image/jpeg")), "jpg");
        assert_eq!(
            media_extension(MediaKind::Video, Some("video/webm")),
            "webm"
        );
        assert_eq!(media_extension(MediaKind::Video, None), "mp4");
    }

    #[test]
    fn content_type_must_match_media_kind() {
        assert!(validate_content_type(MediaKind::Image, Some("image/webp")).is_ok());
        assert!(validate_content_type(MediaKind::Video, Some("video/mp4")).is_ok());
        assert!(matches!(
            validate_content_type(MediaKind::Image, Some("video/mp4")),
            Err(AppError::BadRequest("content_type"))
        ));
        assert!(matches!(
            validate_content_type(MediaKind::Image, None),
            Err(AppError::BadRequest("content_type"))
        ));
    }

    #[test]
    fn upload_size_must_fit_media_kind() {
        assert!(validate_upload_size(MediaKind::Image, Some(MAX_IMAGE_UPLOAD_BYTES)).is_ok());
        assert!(matches!(
            validate_upload_size(MediaKind::Image, Some(MAX_IMAGE_UPLOAD_BYTES + 1)),
            Err(AppError::BadRequest("size_bytes"))
        ));
        assert!(matches!(
            validate_upload_size(MediaKind::Video, Some(0)),
            Err(AppError::BadRequest("size_bytes"))
        ));
    }
}
