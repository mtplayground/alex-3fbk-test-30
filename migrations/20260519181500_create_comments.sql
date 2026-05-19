CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT comments_body_not_empty CHECK (length(trim(body)) > 0),
    CONSTRAINT comments_not_self_parent CHECK (parent_id IS NULL OR parent_id <> id)
);

CREATE INDEX comments_post_created_at_idx ON comments (post_id, created_at ASC);
CREATE INDEX comments_parent_id_idx ON comments (parent_id);
CREATE INDEX comments_author_id_idx ON comments (author_id);
