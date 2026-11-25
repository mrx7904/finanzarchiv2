-- Settings of the archive, one value per key. Defaults live in the core, a row only overrides one.
CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
) STRICT;
