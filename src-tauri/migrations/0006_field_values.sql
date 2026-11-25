-- Every value read from a document with its source (FA-15): the reader, the place on the page and the
-- time. The reader is never empty, a value nobody read does not exist. `value` is the normalized form:
-- whole cents for an amount, an ISO date, otherwise the text.
CREATE TABLE field_values (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents (id),
    field TEXT NOT NULL CHECK (length(field) > 0),
    printed TEXT NOT NULL,
    value TEXT NOT NULL,
    reader TEXT NOT NULL CHECK (length(reader) > 0),
    page INTEGER,
    left REAL,
    bottom REAL,
    right REAL,
    top REAL,
    read_at TEXT NOT NULL
) STRICT;

CREATE INDEX field_values_document ON field_values (document_id, field);

-- The result of each check, for a field or (field NULL) for the whole document.
CREATE TABLE field_checks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents (id),
    field TEXT,
    check_id TEXT NOT NULL CHECK (length(check_id) > 0),
    passed INTEGER NOT NULL CHECK (passed IN (0, 1)),
    detail TEXT,
    checked_at TEXT NOT NULL
) STRICT;

CREATE INDEX field_checks_document ON field_checks (document_id);

-- Which version of the readers last read a document, so a better reader reads older documents again.
ALTER TABLE documents ADD COLUMN read_version INTEGER NOT NULL DEFAULT 0;
