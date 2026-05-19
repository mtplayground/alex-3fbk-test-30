CREATE TABLE auth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    purpose TEXT NOT NULL,
    consumed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT auth_tokens_token_hash_not_empty CHECK (length(trim(token_hash)) > 0),
    CONSTRAINT auth_tokens_purpose_valid CHECK (
        purpose IN ('email_verification', 'password_reset')
    ),
    CONSTRAINT auth_tokens_expire_after_creation CHECK (expires_at > created_at)
);

CREATE UNIQUE INDEX auth_tokens_token_hash_purpose_unique_idx
    ON auth_tokens (token_hash, purpose);
CREATE INDEX auth_tokens_user_purpose_idx ON auth_tokens (user_id, purpose);
CREATE INDEX auth_tokens_active_lookup_idx ON auth_tokens (token_hash, purpose)
    WHERE consumed_at IS NULL;
