-- The stage `model` (#14): a job whose fields the local model is to read waits there, with the
-- German reason in `error`, while Ollama does not answer. SQLite cannot change a CHECK, so the
-- table is built anew with the same rows. No other table refers to it.
CREATE TABLE intake_jobs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog')),
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    stage TEXT NOT NULL CHECK (stage IN ('queued', 'stored', 'text', 'classified', 'read', 'model', 'checked', 'filed', 'waiting', 'failed')),
    document_id INTEGER REFERENCES documents (id),
    known INTEGER NOT NULL DEFAULT 0 CHECK (known IN (0, 1)),
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

INSERT INTO intake_jobs_new (id, source, path, name, stage, document_id, known, error, created_at, updated_at)
SELECT id, source, path, name, stage, document_id, known, error, created_at, updated_at FROM intake_jobs;

DROP TABLE intake_jobs;
ALTER TABLE intake_jobs_new RENAME TO intake_jobs;

CREATE INDEX intake_jobs_open ON intake_jobs (id) WHERE stage NOT IN ('filed', 'waiting', 'failed');
