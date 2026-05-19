CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followee_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    state TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (follower_id, followee_id),
    CONSTRAINT follows_state_valid CHECK (state IN ('accepted', 'pending')),
    CONSTRAINT follows_not_self CHECK (follower_id <> followee_id)
);

CREATE INDEX follows_followee_state_idx ON follows (followee_id, state, created_at DESC);
CREATE INDEX follows_follower_state_idx ON follows (follower_id, state, created_at DESC);
