-- Moves between own accounts (FA-28, #23). A pair is one move: the booking out of one own account
-- and the booking into another. The other half is made by the app (`bookings.made_by`) where the
-- other account has no statement for it. Each pair is a decision of its rule and can be undone.
CREATE TABLE booking_pairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    out_booking INTEGER NOT NULL REFERENCES bookings (id),
    in_booking INTEGER REFERENCES bookings (id),
    rule TEXT NOT NULL CHECK (length(rule) > 0),
    inputs TEXT NOT NULL,
    decided_at TEXT NOT NULL,
    undone_at TEXT
) STRICT;

CREATE INDEX booking_pairs_out ON booking_pairs (out_booking);
CREATE INDEX booking_pairs_in ON booking_pairs (in_booking);

ALTER TABLE bookings ADD COLUMN made_by TEXT;
