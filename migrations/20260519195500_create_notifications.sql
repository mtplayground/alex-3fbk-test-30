CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    actor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id UUID NOT NULL,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT notifications_kind_valid CHECK (
        kind IN ('like', 'comment', 'follow', 'mention', 'dm')
    ),
    CONSTRAINT notifications_target_kind_valid CHECK (
        target_kind IN ('post', 'comment', 'user', 'message', 'conversation')
    ),
    CONSTRAINT notifications_read_after_creation CHECK (
        read_at IS NULL OR read_at >= created_at
    )
);

CREATE INDEX notifications_user_created_at_idx
    ON notifications (user_id, created_at DESC, id DESC);
CREATE INDEX notifications_user_unread_idx
    ON notifications (user_id, created_at DESC)
    WHERE read_at IS NULL;
CREATE INDEX notifications_actor_id_idx ON notifications (actor_id);
