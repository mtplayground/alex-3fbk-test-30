CREATE EXTENSION IF NOT EXISTS pg_trgm;

ALTER TABLE users
ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    setweight(to_tsvector('simple', coalesce(handle, '')), 'A') ||
    setweight(to_tsvector('simple', coalesce(display_name, '')), 'B')
) STORED;

CREATE INDEX users_search_vector_gin_idx ON users USING GIN (search_vector);
CREATE INDEX users_handle_trgm_idx ON users USING GIN (handle gin_trgm_ops);

ALTER TABLE posts
ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    to_tsvector('simple', coalesce(caption, ''))
) STORED;

CREATE INDEX posts_caption_search_vector_gin_idx ON posts USING GIN (search_vector)
    WHERE deleted_at IS NULL;

CREATE INDEX hashtags_name_trgm_idx ON hashtags USING GIN (name gin_trgm_ops);
