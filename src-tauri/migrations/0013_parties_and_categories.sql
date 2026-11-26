-- Parties and their categories (FA-30, #25). A party is who money goes to or comes from, however
-- the bank or a document spells it. Each spelling, folded, is an alias, and an IBAN joins a new
-- spelling to its party. A category belongs to a party; a booking may carry its own.
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE CHECK (length(name) > 0),
    direction TEXT NOT NULL CHECK (direction IN ('in', 'out')),
    position INTEGER NOT NULL
) STRICT;

CREATE TABLE parties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK (length(name) > 0),
    category_id INTEGER REFERENCES categories (id),
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE party_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    party_id INTEGER NOT NULL REFERENCES parties (id),
    folded TEXT NOT NULL UNIQUE CHECK (length(folded) > 0),
    spelling TEXT NOT NULL,
    iban TEXT
) STRICT;

CREATE INDEX party_aliases_iban ON party_aliases (iban) WHERE iban IS NOT NULL;

-- The owner's corrections, which the rules read: a category chosen for one booking, or for all of a party.
CREATE TABLE corrections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL CHECK (kind IN ('booking-category', 'party-category', 'merge', 'split')),
    party_id INTEGER REFERENCES parties (id),
    booking_id INTEGER REFERENCES bookings (id),
    category_id INTEGER REFERENCES categories (id),
    detail TEXT,
    made_at TEXT NOT NULL,
    undone_at TEXT
) STRICT;

ALTER TABLE bookings ADD COLUMN party_id INTEGER REFERENCES parties (id);
ALTER TABLE bookings ADD COLUMN category_id INTEGER REFERENCES categories (id);
ALTER TABLE documents ADD COLUMN party_id INTEGER REFERENCES parties (id);
