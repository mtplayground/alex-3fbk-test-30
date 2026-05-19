CREATE TABLE blocks (
    blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_id, blocked_id),
    CONSTRAINT blocks_no_self_block CHECK (blocker_id <> blocked_id)
);

CREATE INDEX blocks_blocked_id_idx ON blocks (blocked_id);

CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_kind TEXT NOT NULL,
    target_id UUID NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT reports_target_kind_valid CHECK (
        target_kind IN ('user', 'post', 'comment', 'message', 'conversation', 'story', 'reel')
    ),
    CONSTRAINT reports_status_valid CHECK (status IN ('open', 'reviewed', 'dismissed', 'actioned')),
    CONSTRAINT reports_reason_not_empty CHECK (length(trim(reason)) > 0)
);

CREATE INDEX reports_status_created_at_idx ON reports (status, created_at DESC);
CREATE INDEX reports_reporter_id_created_at_idx ON reports (reporter_id, created_at DESC);
CREATE INDEX reports_target_idx ON reports (target_kind, target_id);
