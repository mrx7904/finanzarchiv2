-- Accounts, the statements read into them and their bookings (FA-25, #20). A booking is the only
-- source of money (knowledge/decision-data-model.md). Each IBAN is one account.
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    iban TEXT UNIQUE,
    bank TEXT,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'giro' CHECK (kind IN ('giro', 'savings', 'cash', 'other')),
    opening_cents INTEGER,
    opening_day TEXT,
    opening_source TEXT CHECK (opening_source IN ('statement', 'owner')),
    created_at TEXT NOT NULL
) STRICT;

-- What code read of a statement. `bookings_cents` is the sum of its bookings, so the balance check
-- (`opening + bookings = closing`) can be shown without reading them again.
CREATE TABLE statements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL UNIQUE REFERENCES documents (id),
    account_id INTEGER NOT NULL REFERENCES accounts (id),
    layout TEXT NOT NULL,
    number TEXT,
    opening_day TEXT NOT NULL,
    closing_day TEXT NOT NULL,
    opening_cents INTEGER NOT NULL,
    closing_cents INTEGER NOT NULL,
    bookings_cents INTEGER NOT NULL,
    read_at TEXT NOT NULL
) STRICT;

-- One booking of an account: money in is positive, money out negative, whole cents. Its source is
-- the statement and the place of its first line on the page.
CREATE TABLE bookings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts (id),
    statement_id INTEGER REFERENCES statements (id),
    position INTEGER NOT NULL,
    day TEXT NOT NULL,
    value_day TEXT,
    amount_cents INTEGER NOT NULL,
    kind TEXT NOT NULL,
    other_side TEXT,
    purpose TEXT NOT NULL,
    page INTEGER,
    left REAL,
    bottom REAL,
    right REAL,
    top REAL
) STRICT;

CREATE INDEX bookings_account_day ON bookings (account_id, day);
CREATE INDEX bookings_statement ON bookings (statement_id);
