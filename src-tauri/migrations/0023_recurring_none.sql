-- Recurring payments (FA-38, #34): a found payment the owner said is none. Party, direction,
-- rhythm and the day of its first payment name it, so it is not found again. No foreign key: a
-- merged party leaves its rows behind, which then name nothing.
CREATE TABLE recurring_none (
    party_id INTEGER NOT NULL,
    incoming INTEGER NOT NULL CHECK (incoming IN (0, 1)),
    rhythm TEXT NOT NULL,
    since TEXT NOT NULL,
    decided_at TEXT NOT NULL,
    PRIMARY KEY (party_id, incoming, rhythm, since)
) STRICT;
