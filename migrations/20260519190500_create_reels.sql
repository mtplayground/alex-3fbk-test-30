CREATE TABLE reels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    media_id UUID NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
    caption TEXT NOT NULL DEFAULT '',
    duration_ms BIGINT,
    audio_title TEXT,
    audio_artist TEXT,
    audio_is_original BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT reels_caption_length CHECK (char_length(caption) <= 2200),
    CONSTRAINT reels_duration_positive CHECK (duration_ms IS NULL OR duration_ms > 0),
    CONSTRAINT reels_audio_title_not_empty CHECK (
        audio_title IS NULL OR length(trim(audio_title)) > 0
    ),
    CONSTRAINT reels_audio_artist_not_empty CHECK (
        audio_artist IS NULL OR length(trim(audio_artist)) > 0
    ),
    CONSTRAINT reels_deleted_after_creation CHECK (
        deleted_at IS NULL OR deleted_at >= created_at
    )
);

CREATE UNIQUE INDEX reels_media_id_unique_idx ON reels (media_id);
CREATE INDEX reels_author_id_idx ON reels (author_id);
CREATE INDEX reels_author_recency_idx ON reels (author_id, created_at DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX reels_recency_idx ON reels (created_at DESC)
    WHERE deleted_at IS NULL;
