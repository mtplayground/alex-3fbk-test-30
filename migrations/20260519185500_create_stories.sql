CREATE TABLE stories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    media_id UUID NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '24 hours'),
    CONSTRAINT stories_expires_after_creation CHECK (expires_at > created_at)
);

CREATE INDEX stories_author_id_idx ON stories (author_id);
CREATE INDEX stories_author_expires_at_idx ON stories (author_id, expires_at DESC);
CREATE INDEX stories_expires_at_idx ON stories (expires_at);
CREATE INDEX stories_media_id_idx ON stories (media_id);

CREATE TABLE story_views (
    story_id UUID NOT NULL REFERENCES stories(id) ON DELETE CASCADE,
    viewer_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (story_id, viewer_id)
);

CREATE INDEX story_views_viewer_id_idx ON story_views (viewer_id);
CREATE INDEX story_views_viewed_at_idx ON story_views (viewed_at);
