CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    handle TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL,
    bio TEXT NOT NULL DEFAULT '',
    link TEXT,
    avatar_key TEXT,
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    email_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT users_email_not_empty CHECK (length(trim(email)) > 0),
    CONSTRAINT users_handle_not_empty CHECK (length(trim(handle)) > 0),
    CONSTRAINT users_password_hash_not_empty CHECK (length(trim(password_hash)) > 0),
    CONSTRAINT users_display_name_not_empty CHECK (length(trim(display_name)) > 0)
);

CREATE UNIQUE INDEX users_email_unique_idx ON users (email);
CREATE UNIQUE INDEX users_handle_unique_idx ON users (handle);
