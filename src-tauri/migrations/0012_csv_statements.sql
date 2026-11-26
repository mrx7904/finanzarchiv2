-- CSV statements (#21): a document may be a `csv` file, whose text is a `table`, and a booking that
-- a CSV of the same account holds too is replaced by it (CSV before PDF).
-- SQLite cannot change a CHECK, so `documents` is built anew as SQLite recommends: the runner turns
-- the foreign keys off around the migrations and checks them before the commit
-- (`PRAGMA foreign_key_check`), the new table takes the name the other tables refer to.
CREATE TABLE documents_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL UNIQUE CHECK (length(sha256) = 64),
    file_type TEXT NOT NULL CHECK (file_type IN ('pdf', 'png', 'jpeg', 'webp', 'csv')),
    kind TEXT NOT NULL DEFAULT 'unknown',
    original_name TEXT NOT NULL,
    name TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes > 0),
    pages INTEGER,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog')),
    imported_at TEXT NOT NULL,
    text_state TEXT CHECK (text_state IN ('text', 'scan', 'image', 'table')),
    kind_signals TEXT,
    read_version INTEGER NOT NULL DEFAULT 0,
    attachments_read INTEGER NOT NULL DEFAULT 0 CHECK (attachments_read IN (0, 1)),
    state TEXT NOT NULL DEFAULT 'open' CHECK (state IN ('open', 'filed', 'waiting')),
    name_by_owner INTEGER NOT NULL DEFAULT 0 CHECK (name_by_owner IN (0, 1))
) STRICT;

INSERT INTO documents_new (id, sha256, file_type, kind, original_name, name, size_bytes, pages, source, imported_at,
                       text_state, kind_signals, read_version, attachments_read, state, name_by_owner)
SELECT id, sha256, file_type, kind, original_name, name, size_bytes, pages, source, imported_at,
       text_state, kind_signals, read_version, attachments_read, state, name_by_owner
FROM documents;

DROP TABLE documents;

ALTER TABLE documents_new RENAME TO documents;

ALTER TABLE bookings ADD COLUMN replaced_by INTEGER REFERENCES bookings (id);
