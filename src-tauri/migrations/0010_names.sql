-- Whether the owner typed the name of a document (#18). A name of the owner is kept, a template
-- never writes over it.
ALTER TABLE documents ADD COLUMN name_by_owner INTEGER NOT NULL DEFAULT 0 CHECK (name_by_owner IN (0, 1));
