-- Savings hints (FA-42, #39): a hint the owner dismissed, with the numbers it showed. It comes back
-- when its numbers change.
CREATE TABLE hints_dismissed (
    key TEXT PRIMARY KEY,
    numbers TEXT NOT NULL,
    dismissed_at TEXT NOT NULL
) STRICT;
