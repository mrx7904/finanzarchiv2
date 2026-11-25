-- Every imported file, stored unchanged under its SHA-256 in the archive folder (FA-07).
-- AUTOINCREMENT, so the id of a deleted document is never given again (lesson of 1.0).
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL UNIQUE CHECK (length(sha256) = 64),
    file_type TEXT NOT NULL CHECK (file_type IN ('pdf', 'png', 'jpeg', 'webp')),
    kind TEXT NOT NULL DEFAULT 'unknown',
    original_name TEXT NOT NULL,
    name TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes > 0),
    pages INTEGER,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog')),
    imported_at TEXT NOT NULL
) STRICT;
