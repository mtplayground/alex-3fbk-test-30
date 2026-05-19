CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    caption TEXT NOT NULL DEFAULT '',
    location TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT posts_location_not_empty CHECK (
        location IS NULL OR length(trim(location)) > 0
    ),
    CONSTRAINT posts_deleted_after_creation CHECK (
        deleted_at IS NULL OR deleted_at >= created_at
    )
);

CREATE INDEX posts_author_id_idx ON posts (author_id);
CREATE INDEX posts_author_recency_idx ON posts (author_id, created_at DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX posts_recency_idx ON posts (created_at DESC)
    WHERE deleted_at IS NULL;

CREATE TABLE post_media (
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    media_id UUID NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
    position INTEGER NOT NULL,
    PRIMARY KEY (post_id, media_id),
    CONSTRAINT post_media_position_non_negative CHECK (position >= 0)
);

CREATE UNIQUE INDEX post_media_post_position_unique_idx ON post_media (post_id, position);
CREATE UNIQUE INDEX post_media_media_id_unique_idx ON post_media (media_id);
CREATE INDEX post_media_post_id_idx ON post_media (post_id);

CREATE TABLE hashtags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT hashtags_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT hashtags_name_normalized CHECK (name = lower(name))
);

CREATE UNIQUE INDEX hashtags_name_unique_idx ON hashtags (name);

CREATE TABLE post_hashtags (
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    hashtag_id UUID NOT NULL REFERENCES hashtags(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, hashtag_id)
);

CREATE INDEX post_hashtags_hashtag_id_idx ON post_hashtags (hashtag_id);

CREATE TABLE mentions (
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    mentioned_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    handle TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (post_id, mentioned_user_id, position),
    CONSTRAINT mentions_handle_not_empty CHECK (length(trim(handle)) > 0),
    CONSTRAINT mentions_position_non_negative CHECK (position >= 0)
);

CREATE INDEX mentions_post_id_idx ON mentions (post_id);
CREATE INDEX mentions_mentioned_user_id_idx ON mentions (mentioned_user_id);
