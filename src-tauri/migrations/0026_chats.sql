-- The chat over the archive (FA-45, #42): the history in chats the owner can delete. The answers
-- are stored with their sources, the documents and bookings the lookups returned.
CREATE TABLE chats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL REFERENCES chats (id),
    role TEXT NOT NULL CHECK (role IN ('owner', 'archive')),
    text TEXT NOT NULL,
    sources TEXT NOT NULL DEFAULT '[]',
    asked_at TEXT NOT NULL
) STRICT;

CREATE INDEX chat_messages_chat ON chat_messages (chat_id);
