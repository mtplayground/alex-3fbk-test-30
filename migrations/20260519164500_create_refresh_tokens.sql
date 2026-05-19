CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_jti UUID NOT NULL,
    rotated_from_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    replaced_by_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    revoked_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT refresh_tokens_expire_after_creation CHECK (expires_at > created_at),
    CONSTRAINT refresh_tokens_not_self_rotated CHECK (
        rotated_from_token_id IS NULL OR rotated_from_token_id <> id
    ),
    CONSTRAINT refresh_tokens_not_self_replaced CHECK (
        replaced_by_token_id IS NULL OR replaced_by_token_id <> id
    )
);

CREATE UNIQUE INDEX refresh_tokens_token_jti_unique_idx ON refresh_tokens (token_jti);
CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens (user_id);
CREATE INDEX refresh_tokens_active_token_jti_idx ON refresh_tokens (token_jti)
    WHERE revoked_at IS NULL AND replaced_by_token_id IS NULL;
