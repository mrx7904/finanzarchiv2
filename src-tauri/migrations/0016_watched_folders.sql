-- Watched folders (FA-32, #27) and the files seen in them. A file comes in with the source `folder`
-- (and later `mail`), so `documents` and `intake_jobs` are built anew to allow them. The runner turns
-- the foreign keys off around a migration and checks them before its commit (decision-archive-file.md).
CREATE TABLE watched_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    is_inbox INTEGER NOT NULL DEFAULT 0 CHECK (is_inbox IN (0, 1)),
    recursive INTEGER NOT NULL DEFAULT 1 CHECK (recursive IN (0, 1)),
    added_at TEXT NOT NULL,
    problem TEXT
) STRICT;

CREATE UNIQUE INDEX watched_folders_one_inbox ON watched_folders (is_inbox) WHERE is_inbox = 1;

-- A file once handed to the intake, by path, size and time of change, so a start does not hand it again.
CREATE TABLE seen_files (
    folder_id INTEGER NOT NULL REFERENCES watched_folders (id),
    path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    modified TEXT NOT NULL,
    seen_at TEXT NOT NULL,
    PRIMARY KEY (folder_id, path)
) STRICT;

-- The hash of a document the owner deleted: a watched folder does not bring it back.
CREATE TABLE deleted_sources (
    sha256 TEXT PRIMARY KEY CHECK (length(sha256) = 64),
    deleted_at TEXT NOT NULL
) STRICT;

CREATE TABLE documents_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL UNIQUE CHECK (length(sha256) = 64),
    file_type TEXT NOT NULL CHECK (file_type IN ('pdf', 'png', 'jpeg', 'webp', 'csv')),
    kind TEXT NOT NULL DEFAULT 'unknown',
    original_name TEXT NOT NULL,
    name TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes > 0),
    pages INTEGER,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog', 'folder', 'mail')),
    imported_at TEXT NOT NULL,
    text_state TEXT CHECK (text_state IN ('text', 'scan', 'image', 'table')),
    kind_signals TEXT,
    read_version INTEGER NOT NULL DEFAULT 0,
    attachments_read INTEGER NOT NULL DEFAULT 0 CHECK (attachments_read IN (0, 1)),
    state TEXT NOT NULL DEFAULT 'open' CHECK (state IN ('open', 'filed', 'waiting')),
    name_by_owner INTEGER NOT NULL DEFAULT 0 CHECK (name_by_owner IN (0, 1)),
    party_id INTEGER REFERENCES parties (id)
) STRICT;

INSERT INTO documents_new SELECT id, sha256, file_type, kind, original_name, name, size_bytes, pages, source, imported_at,
    text_state, kind_signals, read_version, attachments_read, state, name_by_owner, party_id FROM documents;
DROP TABLE documents;
ALTER TABLE documents_new RENAME TO documents;

CREATE TABLE intake_jobs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog', 'folder', 'mail')),
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    stage TEXT NOT NULL CHECK (stage IN ('queued', 'stored', 'text', 'classified', 'read', 'model', 'checked', 'filed', 'waiting', 'failed')),
    document_id INTEGER REFERENCES documents (id),
    known INTEGER NOT NULL DEFAULT 0 CHECK (known IN (0, 1)),
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

INSERT INTO intake_jobs_new SELECT id, source, path, name, stage, document_id, known, error, created_at, updated_at FROM intake_jobs;
DROP TABLE intake_jobs;
ALTER TABLE intake_jobs_new RENAME TO intake_jobs;

CREATE INDEX intake_jobs_open ON intake_jobs (id) WHERE stage NOT IN ('filed', 'waiting', 'failed');
