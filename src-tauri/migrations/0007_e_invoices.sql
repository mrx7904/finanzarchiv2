-- The XML of an e-invoice (ZUGFeRD, Factur-X, XRechnung) as its PDF carries it (FA-17, #12). The
-- stage `text` looks for it once per PDF and marks that in `documents.attachments_read`.
CREATE TABLE e_invoices (
    document_id INTEGER PRIMARY KEY REFERENCES documents (id),
    name TEXT NOT NULL,
    xml BLOB NOT NULL
) STRICT;

ALTER TABLE documents ADD COLUMN attachments_read INTEGER NOT NULL DEFAULT 0 CHECK (attachments_read IN (0, 1));

-- The element of the XML a value comes from, as the line is the source of a printed value.
ALTER TABLE field_values ADD COLUMN element TEXT;
