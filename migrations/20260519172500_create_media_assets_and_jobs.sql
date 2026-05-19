CREATE TABLE media_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    original_key TEXT NOT NULL,
    variants JSONB NOT NULL DEFAULT '{}'::jsonb,
    duration_ms BIGINT,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT media_assets_kind_valid CHECK (kind IN ('image', 'video')),
    CONSTRAINT media_assets_status_valid CHECK (
        status IN ('pending', 'uploaded', 'processing', 'ready', 'failed')
    ),
    CONSTRAINT media_assets_original_key_not_empty CHECK (length(trim(original_key)) > 0),
    CONSTRAINT media_assets_variants_is_object CHECK (jsonb_typeof(variants) = 'object'),
    CONSTRAINT media_assets_duration_non_negative CHECK (
        duration_ms IS NULL OR duration_ms >= 0
    ),
    CONSTRAINT media_assets_width_positive CHECK (width IS NULL OR width > 0),
    CONSTRAINT media_assets_height_positive CHECK (height IS NULL OR height > 0)
);

CREATE UNIQUE INDEX media_assets_original_key_unique_idx ON media_assets (original_key);
CREATE INDEX media_assets_owner_id_idx ON media_assets (owner_id);
CREATE INDEX media_assets_status_idx ON media_assets (status);

CREATE TABLE media_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES media_assets(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    run_after TIMESTAMPTZ NOT NULL DEFAULT now(),
    locked_at TIMESTAMPTZ,
    locked_by TEXT,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT media_jobs_kind_valid CHECK (kind IN ('image_processing', 'video_processing')),
    CONSTRAINT media_jobs_status_valid CHECK (status IN ('queued', 'running', 'succeeded', 'failed')),
    CONSTRAINT media_jobs_payload_is_object CHECK (jsonb_typeof(payload) = 'object'),
    CONSTRAINT media_jobs_attempts_non_negative CHECK (attempts >= 0),
    CONSTRAINT media_jobs_max_attempts_positive CHECK (max_attempts > 0)
);

CREATE INDEX media_jobs_asset_id_idx ON media_jobs (asset_id);
CREATE INDEX media_jobs_status_idx ON media_jobs (status);
CREATE INDEX media_jobs_queue_idx ON media_jobs (run_after, created_at)
    WHERE status = 'queued';
