-- One job per file that came in, whatever the way (FA-09). A job moves through fixed stages and
-- keeps the one it reached, so a restart resumes there. `queued` and `failed` frame the stages of
-- knowledge/decision-data-model.md.
CREATE TABLE intake_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL CHECK (source IN ('drop', 'dialog')),
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    stage TEXT NOT NULL CHECK (stage IN ('queued', 'stored', 'text', 'classified', 'read', 'checked', 'filed', 'waiting', 'failed')),
    document_id INTEGER REFERENCES documents (id),
    known INTEGER NOT NULL DEFAULT 0 CHECK (known IN (0, 1)),
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX intake_jobs_open ON intake_jobs (id) WHERE stage NOT IN ('filed', 'waiting', 'failed');
