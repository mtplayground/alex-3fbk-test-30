ALTER TABLE users
    ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN suspended_at TIMESTAMPTZ;

ALTER TABLE comments
    ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE INDEX comments_deleted_at_idx ON comments (deleted_at);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    report_id UUID REFERENCES reports(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_id UUID NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT audit_logs_action_not_empty CHECK (length(trim(action)) > 0),
    CONSTRAINT audit_logs_target_kind_not_empty CHECK (length(trim(target_kind)) > 0)
);

CREATE INDEX audit_logs_admin_created_at_idx ON audit_logs (admin_id, created_at DESC);
CREATE INDEX audit_logs_report_id_idx ON audit_logs (report_id);
