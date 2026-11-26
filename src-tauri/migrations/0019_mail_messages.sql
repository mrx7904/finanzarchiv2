-- Every mail looked at (FA-35, #31): one row under the hash of its Message-ID, with the verdict
-- and no body, so no mail is asked about twice. A row outlives its mailbox, so a mailbox added
-- again does not bring back what was looked at. The mailbox keeps where its last round ended.
CREATE TABLE mail_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mailbox_id INTEGER REFERENCES mailboxes (id) ON DELETE SET NULL,
    message_hash TEXT NOT NULL UNIQUE CHECK (length(message_hash) = 64),
    received_at TEXT,
    looked_at TEXT NOT NULL,
    verdict TEXT NOT NULL,
    taken INTEGER NOT NULL DEFAULT 0 CHECK (taken >= 0)
) STRICT;

ALTER TABLE mailboxes ADD COLUMN folder TEXT;
ALTER TABLE mailboxes ADD COLUMN uid_validity INTEGER;
ALTER TABLE mailboxes ADD COLUMN last_uid INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mailboxes ADD COLUMN read_at TEXT;
