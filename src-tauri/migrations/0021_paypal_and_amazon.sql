-- PayPal and Amazon (FA-36, #32).
-- The transaction code of a PayPal mail looked at, so a second mail of one payment is not taken,
-- and whether the Amazon orders of a mail were read.
ALTER TABLE mail_messages ADD COLUMN paypal_code TEXT;
ALTER TABLE mail_messages ADD COLUMN orders_read INTEGER NOT NULL DEFAULT 0 CHECK (orders_read IN (0, 1));
CREATE INDEX mail_messages_paypal ON mail_messages (paypal_code) WHERE paypal_code IS NOT NULL;

-- A PayPal mail that is the receipt of the seller's invoice. An undone pair stays, so the rule
-- does not make it again.
CREATE TABLE receipts_of (
    receipt_id INTEGER PRIMARY KEY REFERENCES documents (id),
    invoice_id INTEGER NOT NULL REFERENCES documents (id),
    rule TEXT NOT NULL CHECK (length(rule) > 0),
    decided_at TEXT NOT NULL,
    undone_at TEXT
) STRICT;
CREATE INDEX receipts_of_invoice ON receipts_of (invoice_id);

-- An order a shop's mail told (Amazon), and the invoice the owner downloaded for it.
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shop TEXT NOT NULL,
    order_number TEXT NOT NULL UNIQUE,
    ordered_at TEXT,
    total_cents INTEGER,
    mail_hash TEXT,
    document_id INTEGER REFERENCES documents (id) ON DELETE SET NULL,
    found_at TEXT NOT NULL
) STRICT;

-- The mailboxes read their reach once more, so the orders of mails looked at before are found.
-- Every mail looked at is known by its hash and not judged again.
UPDATE mailboxes SET last_uid = 0;
