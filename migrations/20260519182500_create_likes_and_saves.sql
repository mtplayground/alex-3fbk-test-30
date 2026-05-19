CREATE TABLE likes (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, target_kind, target_id),
    CONSTRAINT likes_target_kind_valid CHECK (target_kind IN ('post', 'comment'))
);

CREATE INDEX likes_target_idx ON likes (target_kind, target_id);
CREATE INDEX likes_user_id_idx ON likes (user_id);

CREATE TABLE saves (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, post_id)
);

CREATE INDEX saves_post_id_idx ON saves (post_id);
CREATE INDEX saves_user_id_idx ON saves (user_id);
