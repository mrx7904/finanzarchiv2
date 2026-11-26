-- A mail that is the document itself is kept as its prepared text (FA-35, #31, the owner's word of
-- 2026-11-26), not as a PDF: the file type `mail` and the text state `mail`. SQLite cannot change a
-- CHECK, so `documents` is built anew, as in 0016.
CREATE TABLE documents_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL UNIQUE CHECK (length(sha256) = 64),
    file_type TEXT NOT NULL CHECK (file_type IN ('pdf', 'png', 'jpeg', 'webp', 'csv', 'mail')),
    kind TEXT NOT NULL DEFAULT 'unknown',
    original_name TEXT NOT NULL,
    name TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes > 0),
    pages INTEGER,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog', 'folder', 'mail')),
    imported_at TEXT NOT NULL,
    text_state TEXT CHECK (text_state IN ('text', 'scan', 'image', 'table', 'mail')),
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

-- The kind the model gave a mail it took, the hint for the intake when code cannot tell.
ALTER TABLE mail_messages ADD COLUMN kind TEXT;
UPDATE mail_messages SET kind = CASE
    WHEN verdict LIKE '%laut Modell eine Rechnung%' THEN 'invoice'
    WHEN verdict LIKE '%laut Modell eine Gehaltsabrechnung%' THEN 'payslip'
    WHEN verdict LIKE '%laut Modell ein Zahlungsbeleg%' THEN 'receipt'
    WHEN verdict LIKE '%laut Modell ein Kontoauszug%' THEN 'statement'
END WHERE taken > 0;
