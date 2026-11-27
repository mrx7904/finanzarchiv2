-- Taxes for the return (FA-44, #41). A tax assignment on a document or on a booking, never both for
-- one payment: the category, the share used for work, what someone refunded, and for craftsmen the
-- labour share. The amount that counts is computed from it, nothing is entered twice.
CREATE TABLE tax_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER UNIQUE REFERENCES documents (id),
    booking_id INTEGER UNIQUE REFERENCES bookings (id),
    category TEXT NOT NULL CHECK (length(category) > 0),
    share_percent INTEGER NOT NULL DEFAULT 100 CHECK (share_percent BETWEEN 1 AND 100),
    refunded_cents INTEGER NOT NULL DEFAULT 0 CHECK (refunded_cents >= 0),
    labour_cents INTEGER CHECK (labour_cents >= 0),
    made_at TEXT NOT NULL,
    CHECK ((document_id IS NULL) <> (booking_id IS NULL))
) STRICT;

-- What the owner says of a tax year that no document shows: the way to work and the days at home.
CREATE TABLE tax_profiles (
    year INTEGER PRIMARY KEY,
    commute_km INTEGER NOT NULL DEFAULT 0 CHECK (commute_km >= 0),
    commute_days INTEGER NOT NULL DEFAULT 0 CHECK (commute_days BETWEEN 0 AND 366),
    own_car INTEGER NOT NULL DEFAULT 1 CHECK (own_car IN (0, 1)),
    employer_commute_cents INTEGER NOT NULL DEFAULT 0 CHECK (employer_commute_cents >= 0),
    homeoffice_days INTEGER NOT NULL DEFAULT 0 CHECK (homeoffice_days BETWEEN 0 AND 366),
    changed_at TEXT NOT NULL
) STRICT;
