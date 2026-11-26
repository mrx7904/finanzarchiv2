-- Whether a document is filed or waits for the owner (#16), and every automatic decision with its
-- rule, the values it saw, what it changed and how to undo it (knowledge/decision-data-model.md).
ALTER TABLE documents ADD COLUMN state TEXT NOT NULL DEFAULT 'open' CHECK (state IN ('open', 'filed', 'waiting'));

CREATE TABLE decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents (id),
    rule TEXT NOT NULL CHECK (length(rule) > 0),
    inputs TEXT NOT NULL,
    change TEXT NOT NULL,
    undo TEXT NOT NULL,
    decided_at TEXT NOT NULL,
    undone_at TEXT
) STRICT;

CREATE INDEX decisions_document ON decisions (document_id);
