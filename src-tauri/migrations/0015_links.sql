-- Documents linked to the bookings that paid them (FA-29, #24). A link carries the part of the
-- booking it takes, the rule that made it, the values it saw and whether it was undone.
CREATE TABLE links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents (id),
    booking_id INTEGER NOT NULL REFERENCES bookings (id),
    cents INTEGER NOT NULL,
    rule TEXT NOT NULL CHECK (length(rule) > 0),
    inputs TEXT NOT NULL,
    decided_at TEXT NOT NULL,
    undone_at TEXT
) STRICT;

CREATE INDEX links_document ON links (document_id);
CREATE INDEX links_booking ON links (booking_id);

-- A document for which more than one booking fits waits as a question in "Prüfen".
CREATE TABLE link_questions (
    document_id INTEGER PRIMARY KEY REFERENCES documents (id),
    candidates TEXT NOT NULL,
    asked_at TEXT NOT NULL,
    answered_at TEXT
) STRICT;
