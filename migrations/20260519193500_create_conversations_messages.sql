CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kind TEXT NOT NULL,
    title TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT conversations_kind_valid CHECK (kind IN ('dm', 'group')),
    CONSTRAINT conversations_title_not_empty CHECK (
        title IS NULL OR length(trim(title)) > 0
    )
);

CREATE INDEX conversations_kind_idx ON conversations (kind);

CREATE TABLE conversation_members (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_read_message_id UUID,
    PRIMARY KEY (conversation_id, user_id)
);

CREATE INDEX conversation_members_user_joined_at_idx
    ON conversation_members (user_id, joined_at DESC);

CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL DEFAULT '',
    media_id UUID REFERENCES media_assets(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT messages_body_or_media_present CHECK (
        length(trim(body)) > 0 OR media_id IS NOT NULL
    )
);

CREATE INDEX messages_conversation_created_at_idx
    ON messages (conversation_id, created_at DESC, id DESC);
CREATE INDEX messages_author_id_idx ON messages (author_id);
CREATE INDEX messages_media_id_idx ON messages (media_id)
    WHERE media_id IS NOT NULL;

ALTER TABLE conversation_members
    ADD CONSTRAINT conversation_members_last_read_message_id_fkey
    FOREIGN KEY (last_read_message_id) REFERENCES messages(id) ON DELETE SET NULL;
