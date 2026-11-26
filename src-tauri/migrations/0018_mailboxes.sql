-- The owner's mailboxes (FA-34, #29). Only the address lives here, the app password lies in the
-- keychain of the system. `problem` is the German reason the last login failed, NULL when it worked.
CREATE TABLE mailboxes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL UNIQUE,
    host TEXT NOT NULL,
    added_at TEXT NOT NULL,
    checked_at TEXT,
    problem TEXT
) STRICT;
